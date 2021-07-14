# finql-sqlite

finql-sqlite is an implementation of the finql-data data handler
trait used by the finql crate. It is only useful in connection with 
finql. 

The implementation is based on sqlx, which uses macros to enable
query checking at compile time. In order to achieve this, a valid
database must be specified, otherwise the build will fail.

Therefore, pleas follow the following steps to build the library:

1. change to the directory that contains the `Cargo.toml`file
2. export the database string on the command line with `export DATABASE_URL=sqlite:<absolute path of finql-sqlite>/data/test.db`
3. build the library with `cargo build`

