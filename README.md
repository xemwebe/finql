# finql

Purpose of this library is to provide over time a comprehensive toolbox for
quantitative analysis of financial assets in rust. The project is licensed under
Apache 2.0 or MIT license (see files LICENSE-Apache2.0 and LICENSE-MIT) at the
option of the user.

The goal is to provide a toolbox for pricing various financial products, like
bonds options or maybe even more complex products. In the near term, calculation
of the discounted cash flow value of bonds is in the focus, based on what
information is given by a standard prospect. Building blocks to achieve this
target include time periods (e.g. "3M" or "10Y"), bank holiday calendars,
business day adjustment rules, calculation of year fraction with respect to
typical day count convention methods, roll-out of cash flows, and calculating
valuation and risk figures like internal yield or duration that are useful for
an investor in these products. 

Functionality to calculate of figures like fair values which are primarily
interesting in scenarios where one is fully hedged are not in the initial focus,
since an investor is by definition not fully hedged. Nevertheless, they might be
added later for comparison and estimating market prices.

The library also supports storing data, like market data, e.g. market quote
information, data related to portfolio and transaction management to be able to
support portfolio analysis (e.g. calculation of risk figures), and generic
storage of product details (e.g. bond specification). This is done by defining
data handler traits for various data categories, with concrete implementations
supporting storage in a databases (supporting `sqlite3` and `postgreSQL`) or
in memory (using the in-memory feature of `sqlite3`).

Market data quotes can be fetched automatically from various vendors and stored into a database. 
Fetching the (realtime) quote or a quote history is implemented for the vendors 
[yahoo! finance](https://finance.yahoo.com/),
[alpha vantage](https://www.alphavantage.co/), 
[gurufocus](https://www.gurufocus.com/new_index/) and 
[eodhistoricaldata](https://eodhistoricaldata.com/). Please note that all except yahoo! finance 
require a user token that is only provided after registration with the service. For gurufocus,
this required a paid license.

With version 0.8.x onwards, we us the sqlx crate, which supports compile time checks of SQL
queries. This, however, requires that the environment variable DATABASE_URL is set to the 
appropriate connection string. For Postgres, using a unix socket connection the variable
could be set to something like

```bash
export DATABASE_URL="postgresql:///<dbname>?user=<user>&password=<password>&ssl=false"
```

and for sqlite3 the string would look like

```bash
export DATABASE_URL="sqlite::memory"
```

or 

```bash
export DATABASE_URL="sqlite::///test.db"
```