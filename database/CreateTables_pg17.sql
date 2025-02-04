CREATE TABLE IF NOT EXISTS assets (
                id SERIAL PRIMARY KEY,
                asset_class VARCHAR(20) NOT NULL
            );

CREATE TABLE IF NOT EXISTS currencies (
                    id INTEGER PRIMARY KEY,
                    iso_code CHAR(3) NOT NULL UNIQUE,
                    rounding_digits INT NOT NULL,
                    FOREIGN KEY(id) REFERENCES assets(id)
                );

create table if not exists stocks (
                  id INTEGER primary key,
                  name text not null unique,
                  wkn CHAR(6) unique,
                  isin CHAR(12) unique,
                  note text,
                  foreign key(id) references assets(id)
                );

CREATE TABLE IF NOT EXISTS transactions (
                id SERIAL PRIMARY KEY,
                trans_type TEXT NOT NULL,
                asset_id INTEGER,
                cash_amount FLOAT8 NOT NULL,
                cash_currency_id INT NOT NULL,
                cash_date DATE NOT NULL,
                related_trans INTEGER,
                position FLOAT8,
                note TEXT,
                FOREIGN KEY(asset_id) REFERENCES assets(id),
                FOREIGN KEY(cash_currency_id) REFERENCES currencies(id),
                FOREIGN KEY(related_trans) REFERENCES transactions(id)
            );
CREATE TABLE IF NOT EXISTS ticker (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                asset_id INTEGER NOT NULL,
                source TEXT NOT NULL,
                priority INTEGER NOT NULL,
                currency_id INT NOT NULL,
                factor FLOAT8 NOT NULL DEFAULT 1.0,
                tz TEXT,
                cal TEXT,
                FOREIGN KEY(asset_id) REFERENCES assets(id),
                FOREIGN KEY(currency_id) REFERENCES currencies(id)
            );
CREATE TABLE IF NOT EXISTS quotes (
                id SERIAL PRIMARY KEY,
                ticker_id INTEGER NOT NULL,
                price FLOAT8 NOT NULL,
                time TIMESTAMP WITH TIME ZONE NOT NULL,
                volume FLOAT8,
                FOREIGN KEY(ticker_id) REFERENCES ticker(id) 
            );
CREATE TABLE IF NOT EXISTS objects (
            id TEXT PRIMARY KEY,
            object JSON NOT NULL);
