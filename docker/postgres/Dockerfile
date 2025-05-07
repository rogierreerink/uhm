FROM postgres

ENV PGDATA=/var/uhm/pgdata

USER root
RUN mkdir -p /var/uhm
RUN chmod 775 /var/uhm
RUN chown postgres:postgres /var/uhm
USER postgres
