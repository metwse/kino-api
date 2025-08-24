SET default_tablespace = '';

SET default_table_access_method = heap;

CREATE TABLE public.cards (
    id bigint NOT NULL,
    owner_id bigint NOT NULL,
    deck_id bigint NOT NULL,
    front bigint NOT NULL,
    back bigint[] NOT NULL,
    done_at timestamp without time zone
);
ALTER TABLE public.cards OWNER TO kino;

CREATE TABLE public.decks (
    id bigint NOT NULL,
    owner_id bigint NOT NULL,
    card_count bigint DEFAULT 0 NOT NULL,
    "interval" interval NOT NULL,
    level integer NOT NULL
);
ALTER TABLE public.decks OWNER TO kino;

CREATE TABLE public.extensions (
    id bigint NOT NULL,
    owner_id bigint,
    name character varying(24) NOT NULL,
    data character varying(256) NOT NULL
);
ALTER TABLE public.extensions OWNER TO kino;

CREATE TABLE public.faces (
    id bigint NOT NULL,
    owner_id bigint NOT NULL,
    extension_id bigint NOT NULL,
    data character varying(128)
);
ALTER TABLE public.faces OWNER TO kino;

CREATE TABLE public.users (
    id bigint NOT NULL,
    email character varying(254) NOT NULL,
    google_id text NOT NULL,
    username character varying(24),
    name text,
    picture text
);
ALTER TABLE public.users OWNER TO kino;


COPY public.cards (id, owner_id, deck_id, front, back, done_at) FROM stdin;
\.

COPY public.decks (id, owner_id, card_count, "interval", level) FROM stdin;
\.


-- Built-in extensions.
COPY public.extensions (id, owner_id, name, data) FROM stdin;
0	\N	Word	window.extensions.default.Word
1	\N	Note	window.extensions.default.Note
2	\N	WordNet	window.extensions.default.dictionaries.WN
\.


COPY public.faces (id, owner_id, extension_id, data) FROM stdin;
\.

COPY public.users (id, email, google_id, username, name, picture) FROM stdin;
\.


ALTER TABLE ONLY public.cards
    ADD CONSTRAINT cards_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.decks
    ADD CONSTRAINT decks_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.extensions
    ADD CONSTRAINT extensions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.faces
    ADD CONSTRAINT faces_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


CREATE UNIQUE INDEX users_google_id ON public.users USING btree (google_id) INCLUDE (username) WITH (deduplicate_items='true');

CREATE UNIQUE INDEX users_username ON public.users USING btree (username) WITH (deduplicate_items='true');


ALTER TABLE ONLY public.cards
    ADD CONSTRAINT cards_deck_id_fk FOREIGN KEY (deck_id) REFERENCES public.decks(id) NOT VALID;

ALTER TABLE ONLY public.cards
    ADD CONSTRAINT cards_userd_id_fk FOREIGN KEY (owner_id) REFERENCES public.users(id) NOT VALID;

ALTER TABLE ONLY public.decks
    ADD CONSTRAINT decks_owner_id_fk FOREIGN KEY (owner_id) REFERENCES public.users(id) NOT VALID;

ALTER TABLE ONLY public.faces
    ADD CONSTRAINT faces_extension_id FOREIGN KEY (extension_id) REFERENCES public.extensions(id) NOT VALID;

ALTER TABLE ONLY public.faces
    ADD CONSTRAINT faces_owner_id_fk FOREIGN KEY (owner_id) REFERENCES public.users(id) NOT VALID;
