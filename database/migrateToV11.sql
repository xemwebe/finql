create table if not exists currencies (
                    id INTEGER primary key,
                    iso_code CHAR(3) not null unique,
                    rounding_digits INT not null,
                    foreign key(id) references assets(id)
                );

create table if not exists stocks (
      id INTEGER primary key,
      name text not null unique,
      wkn CHAR(6) unique,
      isin CHAR(12) unique,
      note text,
      foreign key(id) references assets(id)
    );

insert into currencies
	select 
		a.id, 
		a.name as iso_code,
		coalesce (r.digits, 2) as rounding_digits
	from
		assets a
		left join rounding_digits r on a.name = r.currency
	where
		LENGTH(a.name) = 3;

insert into stocks
	select 
		a.id,
      	a.name,
      	a.wkn,
      	a.isin,
      	a.note
	from
		assets a
	where	
		LENGTH(a.name) != 3;


alter table ticker 
add column currency_id INT;

update
	ticker
set
	currency_id = c.id
from
	currencies c
where
	ticker.currency = c.iso_code;

alter table ticker alter column currency_id set
not null;

alter table ticker
  drop column currency;

alter table transactions 
add column cash_currency_id INT;

update
	transactions
set
	cash_currency_id = c.id
from
	currencies c
where
	transactions.cash_currency = c.iso_code;

alter table transactions alter column cash_currency_id set
not null;

alter table transactions
  drop column cash_currency;

drop table rounding_digits;

alter table assets
add column asset_class VARCHAR(20);

update
	assets
set
	asset_class = 'currency'
where
	LENGTH(name) = 3;

update
	assets
set
	asset_class = 'stock'
where
	LENGTH(name) != 3;

alter table assets alter column asset_class set
not null;

alter table assets drop column wkn;
alter table assets drop column isin;
alter table assets drop column note;
alter table assets drop column name;
