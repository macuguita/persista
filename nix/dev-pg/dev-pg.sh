export PGROOT="$PWD/.postgres"
export PGDATA="$PGROOT/data"
export PGPORT=54329

mkdir -p "$PGROOT"
if [ ! -f "$PGDATA/PG_VERSION" ]; then
  echo "Initializing PostgreSQL..."
  initdb -D "$PGDATA" --auth=trust
fi

if ! pg_isready -h localhost -p "$PGPORT" >/dev/null 2>&1; then
  echo "Starting PostgreSQL..."
  pg_ctl \
    -D "$PGDATA" \
    -l "$PGROOT/postgres.log" \
    -o "-p $PGPORT" \
    start
fi

createdb \
  -h localhost \
  -p "$PGPORT" \
  persista \
  >/dev/null 2>&1 || true
  
echo "Postgres data: $PGDATA"