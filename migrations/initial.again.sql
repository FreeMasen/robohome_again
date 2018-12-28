CREATE EXTENSION "uuid-ossp";
/************************
--TYPES
*************************/
CREATE TYPE public.FlipKind AS ENUM (
    'Custom',
    'PreDawn',
    'Sunrise',
    'Dusk',
    'Sunset'
);

CREATE TYPE public.FlipDirection AS ENUM (
    'Off',
    'On'
);

CREATE TYPE public.FlipInfo AS
(
	id INTEGER,
	hour INTEGER,
	minute INTEGER,
    dow INTEGER,
	direction public.FlipDirection,
    kind public.FlipKind
);

CREATE TYPE public.SwitchFlip AS (
    hour INTEGER,
    minute INTEGER,
    code INTEGER
);

CREATE TYPE public.Auth AS (
    created TIMESTAMP WITH TIME ZONE,
    token UUID
);

ALTER TYPE public.FlipKind
    OWNER TO robot;

ALTER TYPE public.FlipDirection
    OWNER TO robot;

ALTER TYPE public.FlipInfo
    OWNER TO robot;

ALTER TYPE public.SwitchFlip
    OWNER TO robot;

ALTER TYPE public.Auth
    OWNER to robot;

--SEQUENCES
CREATE SEQUENCE public.switch_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE SEQUENCE public.flip_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE SEQUENCE public.authorize_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE SEQUENCE public.token_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE SEQUENCE special_time_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.switch_id_seq
    OWNER to robot;

ALTER SEQUENCE public.flip_id_seq
    OWNER to robot;

ALTER SEQUENCE public.authorize_id_seq
    OWNER to robot;

ALTER SEQUENCE public.token_id_seq
    OWNER to robot;

ALTER SEQUENCE public.special_time_id_seq
    OWNER to robot;

/************************
-- TABLES
*************************/
CREATE TABLE public.switch
(
    id INTEGER NOT NULL DEFAULT nextval('switch_id_seq'::regclass),
    name character varying(255) COLLATE pg_catalog."default" NOT NULL,
    on_code INTEGER NOT NULL,
    off_code INTEGER NOT NULL,
    CONSTRAINT switch_pkey PRIMARY KEY (id)
)
WITH (
    OIDS = FALSE
)
TABLESPACE pg_default;

CREATE TABLE public.flip
(
    id INTEGER NOT NULL DEFAULT nextval('flip_id_seq'::regclass),
    switch_id INTEGER NOT NULL,
    hour INTEGER NOT NULL,
    minute INTEGER NOT NULL,
    direction public.FlipDirection NOT NULL,
    dow INTEGER NOT NULL,
    kind public.FlipKind NOT NULL DEFAULT 'Custom',
    CONSTRAINT flip_pkey PRIMARY KEY (id),
    CONSTRAINT switch_flip FOREIGN KEY (switch_id)
        REFERENCES public.switch (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE
)
WITH (
    OIDS = FALSE
)
TABLESPACE pg_default;

CREATE TABLE public.authorize (
    id INTEGER NOT NULL DEFAULT nextval('authorize_id_seq'::regclass),
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    token UUID NOT NULL DEFAULT uuid_generate_v4()
)
WITH (
    OIDS = FALSE
)
TABLESPACE pg_default;

CREATE TABLE public.token (
    id INTEGER NOT NULL DEFAULT nextval('token_id_seq'::regclass),
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_name CHARACTER VARYING (255) NOT NULL,
    token UUID NOT NULL DEFAULT uuid_generate_v4()
)
WITH (
    OIDS = FALSE
)
TABLESPACE pg_default;

CREATE TABLE public.special_time (
    id INTEGER NOT NULL DEFAULT nextval('special_time_id_seq'::regclass),
    date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_DATE,
    kind public.FlipKind NOT NULL,
    hour INT,
    minute INT
);

ALTER TABLE public.switch
    OWNER TO robot;

ALTER TABLE public.flip
    OWNER TO robot;

ALTER TABLE public.authorize
    OWNER TO robot;

ALTER TABLE public.token
    OWNER TO robot;

ALTER TABLE public.special_time
    OWNER TO robot;

/************************
* FUNCTIONS
*************************/
/************************
* CREATE
*************************/
CREATE OR REPLACE FUNCTION public.new_switch(
	arg_name text,
	arg_on INTEGER,
	arg_off INTEGER)
    RETURNS switch
    LANGUAGE 'plpgsql'

    COST 100
    VOLATILE
AS $BODY$
DECLARE ret switch;
BEGIN
	INSERT INTO switch (name, on_code, off_code)
	VALUES (arg_name, arg_on, arg_off)
	RETURNING * INTO ret;
	RETURN ret;
END;
$BODY$;

CREATE OR REPLACE FUNCTION public.new_flip(
    arg_switch INTEGER,
    arg_hour INTEGER,
    arg_minute INTEGER,
    arg_dow INTEGER,
    arg_direction public.FlipDirection,
    arg_kind public.FlipKind)
RETURNS flip
LANGUAGE 'plpgsql'

    COST 100
    VOLATILE
AS $BODY$
DECLARE ret flip;
BEGIN
    INSERT INTO public.flip (
        switch_id, hour, minute, dow, direction, kind)
    VALUES (arg_switch, arg_hour, arg_minute, arg_dow, arg_direction, arg_kind)
    RETURNING * into ret;
    RETURN ret;
END;
$BODY$;

CREATE FUNCTION new_token(arg_user TEXT)
RETURNS UUID
LANGUAGE 'plpgsql'
    COST 100
    VOLATILE
AS $BODY$
DECLARE ret UUID;
BEGIN
    INSERT INTO token (user_name)
        VALUES (arg_user)
    RETURNING token INTO ret;
    RETURN ret;
END;
$BODY$;

ALTER FUNCTION public.new_switch(TEXT, INTEGER, INTEGER)
    OWNER TO robot;

ALTER FUNCTION public.new_flip(INTEGER ,INTEGER ,INTEGER ,INTEGER ,public.FlipDirection, public.FlipKind)
    OWNER TO robot;

ALTER FUNCTION public.new_token(TEXT)
    OWNER TO robot;

/************************
* READ
*************************/
CREATE OR REPLACE FUNCTION public.get_all_switches(
	)
    RETURNS SETOF switch 
    LANGUAGE 'sql'

    COST 100
    VOLATILE 
    ROWS 1000
AS $BODY$
	SELECT id, name, on_code, off_code
	FROM public.switch
$BODY$;

CREATE OR REPLACE FUNCTION public.get_switch_flips(
	arg_switch INTEGER
)
    RETURNS SETOF FlipInfo
    LANGUAGE 'sql'

    COST 100
    VOLATILE
    ROWS 1000
AS $BODY$
	SELECT id, hour, minute, dow, direction, kind
	FROM public.flip
	WHERE switch_id = arg_switch
    ORDER BY hour, minute
$BODY$;

CREATE OR REPLACE FUNCTION public.get_flips_for_day(arg_dow INTEGER)
    RETURNS SETOF public.SwitchFlip
    LANGUAGE 'sql'
    COST 100
    VOLATILE
    ROWS 1000
AS $BODY$
    SELECT f.hour, f.minute,
    CASE WHEN f.direction = 'Off' THEN
        s.off_code
    ELSE
        s.on_code
    END AS code
    FROM public.flip as f
        JOIN public.switch as s
        ON f.switch_id = s.id
    WHERE f.dow & arg_dow > 0;
$BODY$;

CREATE OR REPLACE FUNCTION get_flips_for_minute(arg_hour INTEGER, arg_minute INTEGER, arg_dow INTEGER)
RETURNS SETOF public.SwitchFlip
LANGUAGE 'sql'
    COST 100
    VOLATILE
    ROWS 1000
AS $BODY$
    SELECT f.hour, f.minute,
    CASE WHEN f.direction = 'Off' THEN
        s.off_code
    ELSE
        s.on_code
    END AS code
    FROM public.flip as f
        JOIN public.switch as s
        ON f.switch_id = s.id
    WHERE f.dow & arg_dow > 0
      AND f.hour = arg_hour
      AND f.minute = arg_minute;
$BODY$;



CREATE OR REPLACE FUNCTION get_auth(
    arg_token UUID
) RETURNS TIMESTAMP WITH TIME ZONE
LANGUAGE plpgsql
    COST 100
    VOLATILE
AS $BODY$
DECLARE ret TIMESTAMP WITH TIME ZONE;
BEGIN
    SELECT created
        INTO ret
    FROM public.authorize
    WHERE token = arg_token;
    RETURN ret;
END;
$BODY$;

CREATE OR REPLACE FUNCTION get_token(
    arg_user TEXT
) RETURNS UUID
LANGUAGE plpgsql
    COST 100
    VOLATILE
AS $BODY$
DECLARE ret UUID;
BEGIN
    SELECT token
        INTO ret
    FROM public.token
    WHERE arg_user = public.token.user_name;
END;
$BODY$;


ALTER FUNCTION public.get_all_switches()
    OWNER TO robot;

ALTER FUNCTION public.get_switch_flips(INTEGER)
    OWNER TO robot;

ALTER FUNCTION public.get_flips_for_day(INTEGER)
    OWNER TO robot;

ALTER FUNCTION public.get_flips_for_minute(INTEGER, INTEGER, INTEGER)
    OWNER TO robot;

ALTER FUNCTION public.get_auth(UUID)
    OWNER TO robot;

ALTER FUNCTION public.get_token(TEXT)
    OWNER TO robot;

/************************
* UPDATE
*************************/
CREATE OR REPLACE FUNCTION public.update_switch(
    arg_id INTEGER,
    arg_name TEXT,
    arg_on INTEGER,
    arg_off INTEGER
) RETURNS switch
LANGUAGE plpgsql
    COST 100
    VOLATILE
AS $BODY$
DECLARE ret switch;
BEGIN
    UPDATE switch
    SET name = arg_name,
    on_code = arg_on,
    off_code = arg_off
    WHERE id = arg_id
    RETURNING * into ret;
    RETURN ret;
END;
$BODY$;

CREATE OR REPLACE FUNCTION public.update_flip(
    arg_id INTEGER,
    arg_hour INTEGER,
    arg_minute INTEGER,
    arg_dow INTEGER,
    arg_dir public.FlipDirection,
    arg_kind public.FlipKind
) RETURNS flip
LANGUAGE plpgsql
COST 100
VOLATILE
AS $BODY$
DECLARE ret flip;
BEGIN
    UPDATE flip
    SET hour = arg_hour,
    minute = arg_minute,
    dow = arg_dow,
    direction = arg_dir,
    kind = arg_kind
    WHERE id = arg_id
    RETURNING * into ret;
    RETURN ret;
END;
$BODY$;

CREATE OR REPLACE FUNCTION public.update_special_times(
    predawn_hour INT,
    predawn_min INT,
    sunrise_hour INT,
    sunrise_min INT,
    dusk_hour INT,
    dusk_min INT,
    sunset_hour INT,
    sunset_min INT
) RETURNS INT
LANGUAGE plpgsql
    COST 100
    VOLATILE
AS $BODY$
DECLARE pd_ct INT := 0;
DECLARE d_ct INT := 0;
DECLARE ps_ct INT := 0;
DECLARE s_ct INT := 0;
BEGIN
    INSERT INTO special_time (kind, hour, minute)
        VALUES ('PreDawn', predawn_hour, predawn_min),
               ('Sunrise', sunrise_hour, sunrise_min),
               ('Dusk', dusk_hour, dusk_min),
               ('Sunset', sunset_hour, sunset_min);
    UPDATE flip
    SET hour = predawn_hour,
    minute = predawn_min
    WHERE kind = 'PreDawn';
    GET DIAGNOSTICS pd_ct = ROW_COUNT;

    UPDATE flip
    SET hour = sunrise_hour,
    minute = sunrise_min
    WHERE kind = 'Sunrise';
    GET DIAGNOSTICS d_ct = ROW_COUNT;

    UPDATE flip
    SET hour = dusk_hour,
    minute = dusk_min
    WHERE kind = 'Dusk';
    GET DIAGNOSTICS ps_ct = ROW_COUNT;

    UPDATE flip
    SET hour = sunset_hour,
    minute = sunset_min
    WHERE kind = 'Sunset';
    GET DIAGNOSTICS s_ct = ROW_COUNT;
    RETURN pd_ct + d_ct + ps_ct + s_ct;
END;
$BODY$;

ALTER FUNCTION public.update_switch(INTEGER ,TEXT ,INTEGER ,INTEGER)
    OWNER TO robot;

ALTER FUNCTION public.update_flip(INTEGER, INTEGER, INTEGER, INTEGER, public.FlipDirection, public.FlipKind)
    OWNER TO robot;

/************************
* DELETE
*************************/
CREATE OR REPLACE FUNCTION public.remove_switch(
    arg_id INTEGER
) RETURNS INTEGER
    LANGUAGE plpgsql
    AS $BODY$
DECLARE ret INTEGER;
BEGIN
    DELETE FROM switch WHERE id = arg_id;
    GET DIAGNOSTICS ret = ROW_COUNT;
    RETURN ret;
END;
$BODY$;

CREATE OR REPLACE FUNCTION public.remove_flip(
    arg_id INTEGER
) RETURNS INTEGER
    LANGUAGE plpgsql
    AS $BODY$
DECLARE ret INTEGER;
BEGIN
    DELETE FROM flip WHERE id = arg_id;
    GET DIAGNOSTICS ret = ROW_COUNT;
    RETURN ret;
END;
$BODY$;

ALTER FUNCTION public.remove_switch(INTEGER)
    OWNER TO robot;
ALTER FUNCTION public.remove_flip(INTEGER)
    OWNER TO robot;