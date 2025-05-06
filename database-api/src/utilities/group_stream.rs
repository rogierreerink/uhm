#![allow(dead_code)]

use std::{
    future::Future,
    pin::{pin, Pin},
    task::{ready, Context, Poll},
};

use futures_util::{stream::Peekable, Stream, StreamExt};
use pin_project_lite::pin_project;

pub trait GroupStreamExt<S, K, R, M>: Stream
where
    Self: Sized,
    S: Stream,
    K: PartialEq,
    M: AsyncFn(Group<Self, K>) -> R,
{
    fn group_map(self, key_fn: fn(&Self::Item) -> K, map_fn: M) -> GroupMap<Self, K, R, M> {
        GroupMap {
            stream: self.peekable(),
            key: None,
            key_fn,
            map_fn,
        }
    }
}

impl<S, K, R, M> GroupStreamExt<S, K, R, M> for S
where
    S: Stream,
    K: PartialEq,
    M: AsyncFn(Group<Self, K>) -> R,
{
}

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct GroupMap<S, K, R, M>
    where
        S: Stream,
        K: PartialEq,
        M: AsyncFn(Group<S, K>) -> R,
    {
        #[pin]
        stream: Peekable<S>,
        key: Option<K>,
        key_fn: fn(&S::Item) -> K,
        map_fn: M,
    }
}

impl<S, K, R, M> Stream for GroupMap<S, K, R, M>
where
    S: Stream,
    K: PartialEq,
    M: AsyncFn(Group<S, K>) -> R,
{
    type Item = R;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let next = match ready!(this.stream.as_mut().poll_peek(cx)) {
            Some(next) => next,
            None => return Poll::Ready(None),
        };

        if this.key.is_none() {
            *this.key = Some((this.key_fn)(&next));
        }

        let group = pin!((this.map_fn)(Group {
            iter: this.stream,
            is_polled: false,
            key: &mut this.key,
            key_fn: *this.key_fn,
        }));

        Poll::Ready(Some(ready!(group.poll(cx))))
    }
}

pub struct Group<'a, S, K>
where
    S: Stream,
    K: PartialEq,
{
    iter: Pin<&'a mut Peekable<S>>,
    is_polled: bool,
    key: &'a mut Option<K>,
    key_fn: fn(&S::Item) -> K,
}

impl<'a, S, K> Group<'a, S, K>
where
    S: Stream,
    K: PartialEq,
{
    pub fn key(&self) -> &K {
        self.key.as_ref().unwrap()
    }

    pub async fn peek(&mut self) -> Option<&S::Item> {
        self.iter.as_mut().peek().await
    }
}

impl<S, K> Stream for Group<'_, S, K>
where
    S: Stream,
    K: PartialEq,
{
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.is_polled {
            /* A new group is only created when a next value is known to be
             * available since either the GroupMap or the last poll of the
             * previous Group peeks the stream for the next value and its key.
             */
            self.is_polled = true;
            self.iter.as_mut().poll_next(cx)
        } else {
            let key_fn = self.key_fn;

            let next = match ready!(self.iter.as_mut().poll_peek(cx)) {
                Some(next) => next,
                None => return Poll::Ready(None),
            };

            let next_key = key_fn(&next);

            if *self.key.as_ref().unwrap() == next_key {
                self.iter.as_mut().poll_next(cx)
            } else {
                self.key.replace(next_key);
                Poll::Ready(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::{stream, StreamExt};

    use crate::utilities::group_stream::GroupStreamExt;

    #[tokio::test]
    async fn sub_groups() {
        #[derive(PartialEq, Debug, Clone)]
        struct Input {
            group: usize,
            subgroup: usize,
            value: usize,
        }

        #[derive(PartialEq, Debug)]
        struct Output {
            group: usize,
            subgroups: Vec<OutputSub>,
        }

        #[derive(PartialEq, Debug, Clone)]
        struct OutputSub {
            subgroup: usize,
            values: Vec<usize>,
        }

        let vec = stream::iter(vec![
            Input {
                group: 0,
                subgroup: 0,
                value: 0,
            },
            Input {
                group: 0,
                subgroup: 0,
                value: 1,
            },
            Input {
                group: 0,
                subgroup: 1,
                value: 0,
            },
            Input {
                group: 0,
                subgroup: 1,
                value: 1,
            },
            Input {
                group: 1,
                subgroup: 0,
                value: 0,
            },
            Input {
                group: 1,
                subgroup: 0,
                value: 1,
            },
            Input {
                group: 1,
                subgroup: 1,
                value: 0,
            },
            Input {
                group: 1,
                subgroup: 1,
                value: 1,
            },
        ]);

        #[derive(PartialEq)]
        struct Key {
            group: usize,
        }

        #[derive(PartialEq)]
        struct SubKey {
            subgroup: usize,
        }

        let mut stream = vec.group_map(
            |i| Key { group: i.group },
            async |g| Output {
                group: g.key().group,
                subgroups: g
                    .group_map(
                        |i| SubKey {
                            subgroup: i.subgroup,
                        },
                        async |g| OutputSub {
                            subgroup: g.key().subgroup,
                            values: g.map(|g| g.value).collect().await,
                        },
                    )
                    .collect()
                    .await,
            },
        );

        assert_eq!(
            stream.next().await,
            Some(Output {
                group: 0,
                subgroups: vec![
                    OutputSub {
                        subgroup: 0,
                        values: vec![0, 1]
                    },
                    OutputSub {
                        subgroup: 1,
                        values: vec![0, 1]
                    }
                ]
            })
        );
        assert_eq!(
            stream.next().await,
            Some(Output {
                group: 1,
                subgroups: vec![
                    OutputSub {
                        subgroup: 0,
                        values: vec![0, 1]
                    },
                    OutputSub {
                        subgroup: 1,
                        values: vec![0, 1]
                    }
                ]
            })
        );
        assert_eq!(stream.next().await, None);
    }
}
