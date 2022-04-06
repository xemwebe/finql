--
-- PostgreSQL database dump
--

-- Dumped from database version 13.6
-- Dumped by pg_dump version 13.6

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: EXTENSION pgcrypto; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: assets; Type: TABLE; Schema: public; Owner: qltester
--

CREATE TABLE public.assets (
    id integer NOT NULL,
    asset_class character varying(20) NOT NULL
);


ALTER TABLE public.assets OWNER TO qltester;

--
-- Name: assets_id_seq; Type: SEQUENCE; Schema: public; Owner: qltester
--

CREATE SEQUENCE public.assets_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.assets_id_seq OWNER TO qltester;

--
-- Name: assets_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: qltester
--

ALTER SEQUENCE public.assets_id_seq OWNED BY public.assets.id;


--
-- Name: currencies; Type: TABLE; Schema: public; Owner: qltester
--

CREATE TABLE public.currencies (
    id integer NOT NULL,
    iso_code character(3) NOT NULL,
    rounding_digits integer NOT NULL
);


ALTER TABLE public.currencies OWNER TO qltester;

--
-- Name: objects; Type: TABLE; Schema: public; Owner: qltester
--

CREATE TABLE public.objects (
    id text NOT NULL,
    object json NOT NULL
);


ALTER TABLE public.objects OWNER TO qltester;

--
-- Name: quotes; Type: TABLE; Schema: public; Owner: qltester
--

CREATE TABLE public.quotes (
    id integer NOT NULL,
    ticker_id integer NOT NULL,
    price double precision NOT NULL,
    "time" timestamp with time zone NOT NULL,
    volume double precision
);


ALTER TABLE public.quotes OWNER TO qltester;

--
-- Name: quotes_id_seq; Type: SEQUENCE; Schema: public; Owner: qltester
--

CREATE SEQUENCE public.quotes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.quotes_id_seq OWNER TO qltester;

--
-- Name: quotes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: qltester
--

ALTER SEQUENCE public.quotes_id_seq OWNED BY public.quotes.id;


--
-- Name: stocks; Type: TABLE; Schema: public; Owner: qltester
--

CREATE TABLE public.stocks (
    id integer NOT NULL,
    name text NOT NULL,
    wkn character(6),
    isin character(12),
    note text
);


ALTER TABLE public.stocks OWNER TO qltester;

--
-- Name: ticker; Type: TABLE; Schema: public; Owner: qltester
--

CREATE TABLE public.ticker (
    id integer NOT NULL,
    name text NOT NULL,
    asset_id integer NOT NULL,
    source text NOT NULL,
    priority integer NOT NULL,
    currency_id integer NOT NULL,
    factor double precision DEFAULT 1.0 NOT NULL,
    tz text,
    cal text
);


ALTER TABLE public.ticker OWNER TO qltester;

--
-- Name: ticker_id_seq; Type: SEQUENCE; Schema: public; Owner: qltester
--

CREATE SEQUENCE public.ticker_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.ticker_id_seq OWNER TO qltester;

--
-- Name: ticker_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: qltester
--

ALTER SEQUENCE public.ticker_id_seq OWNED BY public.ticker.id;


--
-- Name: transactions; Type: TABLE; Schema: public; Owner: qltester
--

CREATE TABLE public.transactions (
    id integer NOT NULL,
    trans_type text NOT NULL,
    asset_id integer,
    cash_amount double precision NOT NULL,
    cash_currency_id integer NOT NULL,
    cash_date date NOT NULL,
    related_trans integer,
    "position" double precision,
    note text
);


ALTER TABLE public.transactions OWNER TO qltester;

--
-- Name: transactions_id_seq; Type: SEQUENCE; Schema: public; Owner: qltester
--

CREATE SEQUENCE public.transactions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.transactions_id_seq OWNER TO qltester;

--
-- Name: transactions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: qltester
--

ALTER SEQUENCE public.transactions_id_seq OWNED BY public.transactions.id;


--
-- Name: assets id; Type: DEFAULT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.assets ALTER COLUMN id SET DEFAULT nextval('public.assets_id_seq'::regclass);


--
-- Name: quotes id; Type: DEFAULT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.quotes ALTER COLUMN id SET DEFAULT nextval('public.quotes_id_seq'::regclass);


--
-- Name: ticker id; Type: DEFAULT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.ticker ALTER COLUMN id SET DEFAULT nextval('public.ticker_id_seq'::regclass);


--
-- Name: transactions id; Type: DEFAULT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.transactions ALTER COLUMN id SET DEFAULT nextval('public.transactions_id_seq'::regclass);


--
-- Data for Name: assets; Type: TABLE DATA; Schema: public; Owner: qltester
--

COPY public.assets (id, asset_class) FROM stdin;
1	stock
2	currency
\.


--
-- Data for Name: currencies; Type: TABLE DATA; Schema: public; Owner: qltester
--

COPY public.currencies (id, iso_code, rounding_digits) FROM stdin;
2	EUR	2
\.


--
-- Data for Name: objects; Type: TABLE DATA; Schema: public; Owner: qltester
--

COPY public.objects (id, object) FROM stdin;
test	[{"WeekDay": "Sat"}, {"WeekDay": "Sun"}, {"MovableYearlyDay": {"day": 1, "last": null, "first": null, "month": 1}}, {"EasterOffset": {"last": null, "first": null, "offset": -2}}]
\.


--
-- Data for Name: quotes; Type: TABLE DATA; Schema: public; Owner: qltester
--

COPY public.quotes (id, ticker_id, price, "time", volume) FROM stdin;
\.


--
-- Data for Name: stocks; Type: TABLE DATA; Schema: public; Owner: qltester
--

COPY public.stocks (id, name, wkn, isin, note) FROM stdin;
1	Admiral Group plc	AODJ58	GB00B02J6398	Here are my notes
\.


--
-- Data for Name: ticker; Type: TABLE DATA; Schema: public; Owner: qltester
--

COPY public.ticker (id, name, asset_id, source, priority, currency_id, factor, tz, cal) FROM stdin;
\.


--
-- Data for Name: transactions; Type: TABLE DATA; Schema: public; Owner: qltester
--

COPY public.transactions (id, trans_type, asset_id, cash_amount, cash_currency_id, cash_date, related_trans, "position", note) FROM stdin;
1	c	\N	10000	2	2020-01-15	\N	\N	start capital
2	a	1	-9000	2	2020-01-15	\N	10	\N
3	f	\N	-30	2	2020-01-15	2	\N	\N
4	d	1	90	2	2020-01-30	\N	\N	\N
5	t	\N	-40	2	2020-01-30	4	\N	\N
\.


--
-- Name: assets_id_seq; Type: SEQUENCE SET; Schema: public; Owner: qltester
--

SELECT pg_catalog.setval('public.assets_id_seq', 2, true);


--
-- Name: quotes_id_seq; Type: SEQUENCE SET; Schema: public; Owner: qltester
--

SELECT pg_catalog.setval('public.quotes_id_seq', 1, false);


--
-- Name: ticker_id_seq; Type: SEQUENCE SET; Schema: public; Owner: qltester
--

SELECT pg_catalog.setval('public.ticker_id_seq', 1, false);


--
-- Name: transactions_id_seq; Type: SEQUENCE SET; Schema: public; Owner: qltester
--

SELECT pg_catalog.setval('public.transactions_id_seq', 5, true);


--
-- Name: assets assets_pkey; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.assets
    ADD CONSTRAINT assets_pkey PRIMARY KEY (id);


--
-- Name: currencies currencies_iso_code_key; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.currencies
    ADD CONSTRAINT currencies_iso_code_key UNIQUE (iso_code);


--
-- Name: currencies currencies_pkey; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.currencies
    ADD CONSTRAINT currencies_pkey PRIMARY KEY (id);


--
-- Name: objects objects_pkey; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.objects
    ADD CONSTRAINT objects_pkey PRIMARY KEY (id);


--
-- Name: quotes quotes_pkey; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.quotes
    ADD CONSTRAINT quotes_pkey PRIMARY KEY (id);


--
-- Name: stocks stocks_isin_key; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.stocks
    ADD CONSTRAINT stocks_isin_key UNIQUE (isin);


--
-- Name: stocks stocks_name_key; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.stocks
    ADD CONSTRAINT stocks_name_key UNIQUE (name);


--
-- Name: stocks stocks_pkey; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.stocks
    ADD CONSTRAINT stocks_pkey PRIMARY KEY (id);


--
-- Name: stocks stocks_wkn_key; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.stocks
    ADD CONSTRAINT stocks_wkn_key UNIQUE (wkn);


--
-- Name: ticker ticker_pkey; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.ticker
    ADD CONSTRAINT ticker_pkey PRIMARY KEY (id);


--
-- Name: transactions transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_pkey PRIMARY KEY (id);


--
-- Name: currencies currencies_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.currencies
    ADD CONSTRAINT currencies_id_fkey FOREIGN KEY (id) REFERENCES public.assets(id);


--
-- Name: quotes quotes_ticker_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.quotes
    ADD CONSTRAINT quotes_ticker_id_fkey FOREIGN KEY (ticker_id) REFERENCES public.ticker(id);


--
-- Name: stocks stocks_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.stocks
    ADD CONSTRAINT stocks_id_fkey FOREIGN KEY (id) REFERENCES public.assets(id);


--
-- Name: ticker ticker_asset_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.ticker
    ADD CONSTRAINT ticker_asset_id_fkey FOREIGN KEY (asset_id) REFERENCES public.assets(id);


--
-- Name: ticker ticker_currency_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.ticker
    ADD CONSTRAINT ticker_currency_id_fkey FOREIGN KEY (currency_id) REFERENCES public.currencies(id);


--
-- Name: transactions transactions_asset_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_asset_id_fkey FOREIGN KEY (asset_id) REFERENCES public.assets(id);


--
-- Name: transactions transactions_cash_currency_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_cash_currency_id_fkey FOREIGN KEY (cash_currency_id) REFERENCES public.currencies(id);


--
-- Name: transactions transactions_related_trans_fkey; Type: FK CONSTRAINT; Schema: public; Owner: qltester
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_related_trans_fkey FOREIGN KEY (related_trans) REFERENCES public.transactions(id);


--
-- PostgreSQL database dump complete
--

