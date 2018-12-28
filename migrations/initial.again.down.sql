/************************
* FUNCTIONS
*************************/
/************************
* CREATE
*************************/
DROP FUNCTION public.new_switch(TEXT,INTEGER,INTEGER);
DROP FUNCTION public.new_flip(INTEGER,INTEGER,INTEGER,INTEGER, public.FlipDirection, public.FlipKind);
DROP FUNCTION public.new_token(TEXT);
/************************
* READ
*************************/
DROP FUNCTION public.get_all_switches();
DROP FUNCTION public.get_switch_flips(INTEGER);
DROP FUNCTION public.get_flips_for_day(INTEGER);
DROP FUNCTION get_flips_for_minute(INTEGER, INTEGER, INTEGER);
DROP FUNCTION get_auth(UUID);
DROP FUNCTION get_token(TEXT);
/************************
* UPDATE
*************************/
DROP FUNCTION public.update_switch(INTEGER,TEXT,INTEGER,INTEGER);
DROP FUNCTION public.update_flip(INTEGER,INTEGER,INTEGER,INTEGER, public.FlipDirection, public.FlipKind);
/************************
* DELETE
*************************/
DROP FUNCTION public.remove_switch(INTEGER);
DROP FUNCTION public.remove_flip(INTEGER);
DROP FUNCTION update_special_times(INTEGER, INTEGER, INTEGER, INTEGER, INTEGER, INTEGER, INTEGER, INTEGER);
/************************
-- TABLES
*************************/
DROP TABLE public.flip;
DROP TABLE public.switch;
DROP TABLE public.authorize;
DROP TABLE public.token;
DROP TABLE public.special_time;
/************************
--TYPES
*************************/
DROP TYPE public.FlipInfo;
DROP TYPE public.SwitchFlip;
DROP TYPE public.FlipDirection;
DROP TYPE public.FlipKind;
DROP TYPE public.Auth;
/************************
--SEQUENCES
*************************/
DROP SEQUENCE public.switch_id_seq;
DROP SEQUENCE public.flip_id_seq;
DROP SEQUENCE public.authorize_id_seq;
DROP SEQUENCE public.token_id_seq;
DROP SEQUENCE public.special_time_id_seq;
/************************
--EXTENSIONS
*************************/
DROP EXTENSION "uuid-ossp";