#![allow(dead_code)]

use std::iter::Peekable;

pub trait GroupIterExt<I, K, R>: Iterator
where
    Self: Sized,
    I: Iterator,
    K: PartialEq,
{
    fn group_map(
        self,
        key_fn: fn(&Self::Item) -> K,
        map_fn: fn(Group<Self, K>) -> R,
    ) -> GroupMap<Self, K, R> {
        GroupMap {
            iter: self.peekable(),
            key: None,
            key_fn,
            map_fn,
        }
    }
}

impl<I, K, R> GroupIterExt<I, K, R> for I
where
    I: Iterator,
    K: PartialEq,
{
}

pub struct GroupMap<I, K, R>
where
    I: Iterator,
    K: PartialEq,
{
    iter: Peekable<I>,
    key: Option<K>,
    key_fn: fn(&I::Item) -> K,
    map_fn: fn(Group<'_, I, K>) -> R,
}

impl<I, K, R> Iterator for GroupMap<I, K, R>
where
    I: Iterator,
    K: PartialEq,
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.peek()?;

        if self.key.is_none() {
            self.key = Some((self.key_fn)(&next));
        }

        Some((self.map_fn)(Group {
            iter: &mut self.iter,
            is_polled: false,
            key: &mut self.key,
            key_fn: self.key_fn,
        }))
    }
}

pub struct Group<'a, I, K>
where
    I: Iterator,
    K: PartialEq,
{
    iter: &'a mut Peekable<I>,
    is_polled: bool,
    key: &'a mut Option<K>,
    key_fn: fn(&I::Item) -> K,
}

impl<'a, I, K> Group<'a, I, K>
where
    I: Iterator,
    K: PartialEq,
{
    pub fn key(&self) -> &K {
        self.key.as_ref().unwrap()
    }
}

impl<I, K> Iterator for Group<'_, I, K>
where
    I: Iterator,
    K: PartialEq,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        /* A new group is only created when a next value is known to be
         * available, since either the GroupMap or the last poll of the
         * previous Group peeks the iterator for the next value and its
         * key.
         */
        if self.is_polled == false {
            self.is_polled = true;
            self.iter.next()
        } else {
            let next = self.iter.peek()?;
            let next_key = (self.key_fn)(&next);

            if *self.key.as_ref().unwrap() == next_key {
                self.iter.next()
            } else {
                *self.key = Some(next_key);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GroupIterExt;

    #[test]
    fn primitives() {
        #[derive(PartialEq, Debug)]
        struct Output {
            key: usize,
            sum: usize,
        }

        let vec = vec![1, 1, 2, 3, 3, 3, 4, 4];
        let mut iter = vec.iter().group_map(
            |i| **i,
            |g| Output {
                key: *g.key(),
                sum: g.sum(),
            },
        );

        assert_eq!(iter.next(), Some(Output { key: 1, sum: 2 }));
        assert_eq!(iter.next(), Some(Output { key: 2, sum: 2 }));
        assert_eq!(iter.next(), Some(Output { key: 3, sum: 9 }));
        assert_eq!(iter.next(), Some(Output { key: 4, sum: 8 }));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn structs() {
        #[derive(PartialEq, Debug, Clone)]
        struct Input {
            group: usize,
            unique: usize,
        }

        #[derive(PartialEq, Debug)]
        struct Output {
            group: usize,
            unique: Vec<usize>,
        }

        let vec = vec![
            Input {
                group: 123,
                unique: 9,
            },
            Input {
                group: 123,
                unique: 8,
            },
            Input {
                group: 234,
                unique: 7,
            },
            Input {
                group: 234,
                unique: 6,
            },
            Input {
                group: 345,
                unique: 5,
            },
        ];
        let mut iter = vec.iter().group_map(
            |i| i.group,
            |g| Output {
                group: *g.key(),
                unique: g.map(|i| i.unique).collect(),
            },
        );

        assert_eq!(
            iter.next(),
            Some(Output {
                group: 123,
                unique: vec![9, 8]
            })
        );
        assert_eq!(
            iter.next(),
            Some(Output {
                group: 234,
                unique: vec![7, 6]
            })
        );
        assert_eq!(
            iter.next(),
            Some(Output {
                group: 345,
                unique: vec![5]
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn sub_groups() {
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

        let vec = vec![
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
        ];
        let mut iter = vec.iter().group_map(
            |i| i.group,
            |g| Output {
                group: *g.key(),
                subgroups: g
                    .group_map(
                        |i| i.subgroup,
                        |g| OutputSub {
                            subgroup: *g.key(),
                            values: g.map(|g| g.value).collect(),
                        },
                    )
                    .collect(),
            },
        );

        assert_eq!(
            iter.next(),
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
            iter.next(),
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
        assert_eq!(iter.next(), None);
    }
}
