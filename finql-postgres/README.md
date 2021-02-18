# finql-postgres

finql-postgres is an implementation of the finql-data data handler
trait used by the finql crate. The implementation an adaptor
to the postgreSQL database. 
It is only useful in connection with finql. 

The implementation is based on sqlx, which uses macros to enable
query checking at compile time. In order to achieve this, a valid
database must be specified, otherwise the build will fail.

Therefore, pleas follow the following steps to build the library:

1. Setup a postgreSQL server, e.g. following the documentation on https://www.postgresql.org
2. Setup a postgreSQL user named `finqltester`
3. Upload the file `data/finqlpg.sql` to a database of your choice, e.g. by

```bash
psql <databasename> < data/finqlpg.sql
``` 
as some user with write to create new databases, PostgreSQL's default user
`postgres`. 

4. export the database connection string on the command line with
   
```bash
export DATABASE_URL="postgresql://127.0.0.1/<databasename>?user=finqltester&password=<password>&ssl=false"
``` 

for a http connection or  
   
```bash
export DATABASE_URL="postgresql:///<databasename>?user=finqltester&password=<password>&ssl=false"
``` 

for a connection 
via UNIX socket, depending on your setup.

5. build the library with `cargo build`

Please note that this database is only used once for building the library 
and performing all the compile time checks. Once the build is complete, 
the database handler is able to set up a new empty database.
