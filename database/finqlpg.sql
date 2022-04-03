--
-- PostgreSQL database dump
--

-- Dumped from database version 13.1
-- Dumped by pg_dump version 13.1

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

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: assets; Type: TABLE; Schema: public; Owner: finqltester
--

CREATE TABLE public.assets (
    id integer NOT NULL,
    name text NOT NULL,
    wkn text,
    isin text,
    note text
);


ALTER TABLE public.assets OWNER TO finqltester;

--
-- Name: assets_id_seq; Type: SEQUENCE; Schema: public; Owner: finqltester
--

CREATE SEQUENCE public.assets_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.assets_id_seq OWNER TO finqltester;

--
-- Name: assets_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: finqltester
--

ALTER SEQUENCE public.assets_id_seq OWNED BY public.assets.id;


--
-- Name: quotes; Type: TABLE; Schema: public; Owner: finqltester
--

CREATE TABLE public.quotes (
    id integer NOT NULL,
    ticker_id integer NOT NULL,
    price double precision NOT NULL,
    "time" timestamp with time zone NOT NULL,
    volume double precision
);


ALTER TABLE public.quotes OWNER TO finqltester;

--
-- Name: quotes_id_seq; Type: SEQUENCE; Schema: public; Owner: finqltester
--

CREATE SEQUENCE public.quotes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.quotes_id_seq OWNER TO finqltester;

--
-- Name: quotes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: finqltester
--

ALTER SEQUENCE public.quotes_id_seq OWNED BY public.quotes.id;


--
-- Name: rounding_digits; Type: TABLE; Schema: public; Owner: finqltester
--

CREATE TABLE public.rounding_digits (
    id integer NOT NULL,
    currency text NOT NULL,
    digits integer NOT NULL
);


ALTER TABLE public.rounding_digits OWNER TO finqltester;

--
-- Name: rounding_digits_id_seq; Type: SEQUENCE; Schema: public; Owner: finqltester
--

CREATE SEQUENCE public.rounding_digits_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.rounding_digits_id_seq OWNER TO finqltester;

--
-- Name: rounding_digits_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: finqltester
--

ALTER SEQUENCE public.rounding_digits_id_seq OWNED BY public.rounding_digits.id;


--
-- Name: ticker; Type: TABLE; Schema: public; Owner: finqltester
--

CREATE TABLE public.ticker (
    id integer NOT NULL,
    name text NOT NULL,
    asset_id integer NOT NULL,
    source text NOT NULL,
    priority integer NOT NULL,
    currency text NOT NULL,
    factor double precision DEFAULT 1.0 NOT NULL
);


ALTER TABLE public.ticker OWNER TO finqltester;

--
-- Name: ticker_id_seq; Type: SEQUENCE; Schema: public; Owner: finqltester
--

CREATE SEQUENCE public.ticker_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.ticker_id_seq OWNER TO finqltester;

--
-- Name: ticker_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: finqltester
--

ALTER SEQUENCE public.ticker_id_seq OWNED BY public.ticker.id;


--
-- Name: transactions; Type: TABLE; Schema: public; Owner: finqltester
--

CREATE TABLE public.transactions (
    id integer NOT NULL,
    trans_type text NOT NULL,
    asset_id integer,
    cash_amount double precision NOT NULL,
    cash_currency text NOT NULL,
    cash_date date NOT NULL,
    related_trans integer,
    "position" double precision,
    note text
);


ALTER TABLE public.transactions OWNER TO finqltester;

--
-- Name: transactions_id_seq; Type: SEQUENCE; Schema: public; Owner: finqltester
--

CREATE SEQUENCE public.transactions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.transactions_id_seq OWNER TO finqltester;

--
-- Name: transactions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: finqltester
--

ALTER SEQUENCE public.transactions_id_seq OWNED BY public.transactions.id;


--
-- Name: assets id; Type: DEFAULT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.assets ALTER COLUMN id SET DEFAULT nextval('public.assets_id_seq'::regclass);


--
-- Name: quotes id; Type: DEFAULT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.quotes ALTER COLUMN id SET DEFAULT nextval('public.quotes_id_seq'::regclass);


--
-- Name: rounding_digits id; Type: DEFAULT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.rounding_digits ALTER COLUMN id SET DEFAULT nextval('public.rounding_digits_id_seq'::regclass);


--
-- Name: ticker id; Type: DEFAULT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.ticker ALTER COLUMN id SET DEFAULT nextval('public.ticker_id_seq'::regclass);


--
-- Name: transactions id; Type: DEFAULT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.transactions ALTER COLUMN id SET DEFAULT nextval('public.transactions_id_seq'::regclass);


--
-- Data for Name: assets; Type: TABLE DATA; Schema: public; Owner: finqltester
--

COPY public.assets (id, name, wkn, isin, note) FROM stdin;
1	BASF AG	\N	\N	\N
2	Siemens AG	\N	\N	\N
3	BHP Inc.	\N	\N	\N
4	AUS	\N	\N	\N
5	EUR	\N	\N	\N
\.


--
-- Data for Name: quotes; Type: TABLE DATA; Schema: public; Owner: finqltester
--

COPY public.quotes (id, ticker_id, price, "time", volume) FROM stdin;
1	1	67.35	2019-12-30 20:00:00+01	\N
2	1	68.29	2020-01-02 20:00:00+01	\N
3	1	67.27	2020-01-03 20:00:00+01	\N
4	1	66.27	2020-01-06 20:00:00+01	\N
5	1	66.3	2020-01-07 20:00:00+01	\N
7	4	0.9	2020-01-04 00:00:00+01	\N
8	5	1.1111111111111112	2020-01-04 00:00:00+01	\N
\.


--
-- Data for Name: rounding_digits; Type: TABLE DATA; Schema: public; Owner: finqltester
--

COPY public.rounding_digits (id, currency, digits) FROM stdin;
1	XXX	3
\.


--
-- Data for Name: ticker; Type: TABLE DATA; Schema: public; Owner: finqltester
--

COPY public.ticker (id, name, asset_id, source, priority, currency, factor) FROM stdin;
1	BAS.DE	1	yahoo	10	EUR	1
2	SIE.DE	2	yahoo	10	EUR	1
4	AUS/EUR	4	manual	10	EUR	1
5	EUR/AUS	5	manual	10	AUS	1
\.


--
-- Data for Name: transactions; Type: TABLE DATA; Schema: public; Owner: finqltester
--

COPY public.transactions (id, trans_type, asset_id, cash_amount, cash_currency, cash_date, related_trans, "position", note) FROM stdin;
\.


--
-- Name: assets_id_seq; Type: SEQUENCE SET; Schema: public; Owner: finqltester
--

SELECT pg_catalog.setval('public.assets_id_seq', 5, true);


--
-- Name: quotes_id_seq; Type: SEQUENCE SET; Schema: public; Owner: finqltester
--

SELECT pg_catalog.setval('public.quotes_id_seq', 8, true);


--
-- Name: rounding_digits_id_seq; Type: SEQUENCE SET; Schema: public; Owner: finqltester
--

SELECT pg_catalog.setval('public.rounding_digits_id_seq', 1, true);


--
-- Name: ticker_id_seq; Type: SEQUENCE SET; Schema: public; Owner: finqltester
--

SELECT pg_catalog.setval('public.ticker_id_seq', 5, true);


--
-- Name: transactions_id_seq; Type: SEQUENCE SET; Schema: public; Owner: finqltester
--

SELECT pg_catalog.setval('public.transactions_id_seq', 1, false);


--
-- Name: assets assets_isin_key; Type: CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.assets
    ADD CONSTRAINT assets_isin_key UNIQUE (isin);


--
-- Name: assets assets_name_key; Type: CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.assets
    ADD CONSTRAINT assets_name_key UNIQUE (name);


--
-- Name: assets assets_pkey; Type: CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.assets
    ADD CONSTRAINT assets_pkey PRIMARY KEY (id);


--
-- Name: assets assets_wkn_key; Type: CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.assets
    ADD CONSTRAINT assets_wkn_key UNIQUE (wkn);


--
-- Name: quotes quotes_pkey; Type: CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.quotes
    ADD CONSTRAINT quotes_pkey PRIMARY KEY (id);


--
-- Name: rounding_digits rounding_digits_currency_key; Type: CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.rounding_digits
    ADD CONSTRAINT rounding_digits_currency_key UNIQUE (currency);


--
-- Name: rounding_digits rounding_digits_pkey; Type: CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.rounding_digits
    ADD CONSTRAINT rounding_digits_pkey PRIMARY KEY (id);


--
-- Name: ticker ticker_pkey; Type: CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.ticker
    ADD CONSTRAINT ticker_pkey PRIMARY KEY (id);


--
-- Name: transactions transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_pkey PRIMARY KEY (id);


--
-- Name: quotes quotes_ticker_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.quotes
    ADD CONSTRAINT quotes_ticker_id_fkey FOREIGN KEY (ticker_id) REFERENCES public.ticker(id);


--
-- Name: ticker ticker_asset_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.ticker
    ADD CONSTRAINT ticker_asset_id_fkey FOREIGN KEY (asset_id) REFERENCES public.assets(id);


--
-- Name: transactions transactions_asset_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_asset_id_fkey FOREIGN KEY (asset_id) REFERENCES public.assets(id);


--
-- Name: transactions transactions_related_trans_fkey; Type: FK CONSTRAINT; Schema: public; Owner: finqltester
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_related_trans_fkey FOREIGN KEY (related_trans) REFERENCES public.transactions(id);


--
-- PostgreSQL database dump complete
--

