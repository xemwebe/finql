-- Every asset class has exactly one table holding its class specific data.
-- To add a new class (e.g. 'etf'):
--   1. create a table with the same pattern as 'stocks' below,
--   2. insert the class into asset_classes,
--   3. extend the integrity triggers (see asset_integrity.sql).
CREATE TABLE IF NOT EXISTS asset_classes (
                asset_class VARCHAR(20) PRIMARY KEY
            );

INSERT INTO asset_classes (asset_class)
    VALUES ('currency'), ('stock')
    ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS assets (
                id SERIAL PRIMARY KEY,
                asset_class VARCHAR(20) NOT NULL REFERENCES asset_classes(asset_class),
                UNIQUE (id, asset_class)
            );

-- The (id, asset_class) foreign key together with the fixed asset_class column
-- guarantees that a row in a subtype table can only belong to an asset of that
-- very class, and that an asset belongs to at most one subtype table.
CREATE TABLE IF NOT EXISTS currencies (
                    id INTEGER PRIMARY KEY,
                    asset_class VARCHAR(20) NOT NULL DEFAULT 'currency' CHECK (asset_class = 'currency'),
                    iso_code CHAR(3) NOT NULL UNIQUE,
                    rounding_digits INT NOT NULL,
                    CONSTRAINT currencies_asset_fkey FOREIGN KEY(id, asset_class)
                        REFERENCES assets(id, asset_class) DEFERRABLE INITIALLY IMMEDIATE
                );

CREATE TABLE IF NOT EXISTS stocks (
                  id INTEGER PRIMARY KEY,
                  asset_class VARCHAR(20) NOT NULL DEFAULT 'stock' CHECK (asset_class = 'stock'),
                  name TEXT NOT NULL UNIQUE,
                  wkn CHAR(6) UNIQUE,
                  isin CHAR(12) UNIQUE,
                  note TEXT,
                  CONSTRAINT stocks_asset_fkey FOREIGN KEY(id, asset_class)
                      REFERENCES assets(id, asset_class) DEFERRABLE INITIALLY IMMEDIATE
                );

-- Make sure every asset has a row in the table of its asset class.
-- (Copy of asset_integrity.sql)
CREATE OR REPLACE FUNCTION check_asset_has_subtype() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    aid INTEGER;
    cls VARCHAR(20);
    has_subtype BOOLEAN;
BEGIN
    IF TG_OP = 'INSERT' THEN
        aid := NEW.id;
    ELSE
        aid := OLD.id;
    END IF;

    SELECT asset_class INTO cls FROM assets WHERE id = aid;
    IF NOT FOUND THEN
        -- the asset has been removed again within the same transaction
        RETURN NULL;
    END IF;

    has_subtype := CASE cls
        WHEN 'currency' THEN EXISTS (SELECT 1 FROM currencies WHERE id = aid)
        WHEN 'stock'    THEN EXISTS (SELECT 1 FROM stocks WHERE id = aid)
        ELSE FALSE
    END;
    IF NOT has_subtype THEN
        RAISE EXCEPTION 'asset % of class "%" has no matching row in its asset class table', aid, cls
            USING ERRCODE = 'foreign_key_violation';
    END IF;
    RETURN NULL;
END $$;

DROP TRIGGER IF EXISTS assets_require_subtype ON assets;
CREATE CONSTRAINT TRIGGER assets_require_subtype
    AFTER INSERT ON assets
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION check_asset_has_subtype();

DROP TRIGGER IF EXISTS currencies_keep_subtype ON currencies;
CREATE CONSTRAINT TRIGGER currencies_keep_subtype
    AFTER DELETE OR UPDATE OF id ON currencies
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION check_asset_has_subtype();

DROP TRIGGER IF EXISTS stocks_keep_subtype ON stocks;
CREATE CONSTRAINT TRIGGER stocks_keep_subtype
    AFTER DELETE OR UPDATE OF id ON stocks
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION check_asset_has_subtype();

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
