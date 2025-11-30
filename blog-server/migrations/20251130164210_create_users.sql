CREATE TABLE IF NOT EXISTS public.users
(
    id            UUID                     NOT NULL
    CONSTRAINT users_pk
    PRIMARY KEY,
    username      VARCHAR                  NOT NULL,
    email         VARCHAR                  NOT NULL,
    password_hash VARCHAR                  NOT NULL,
    last_login    TIMESTAMP WITH TIME ZONE,
    created_by    TIMESTAMP WITH TIME ZONE NOT NULL,
    deleted_at    TIMESTAMP WITH TIME ZONE
);

CREATE UNIQUE INDEX IF NOT EXISTS users_email_uindex
    ON public.users (email);

CREATE UNIQUE INDEX IF NOT EXISTS users_username_uindex
    ON public.users (username);

