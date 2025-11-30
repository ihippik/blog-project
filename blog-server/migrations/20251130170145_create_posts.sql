CREATE TABLE IF NOT EXISTS public.posts
(
    id            UUID                     NOT NULL
        CONSTRAINT posts_pk
            PRIMARY KEY,
    title      VARCHAR                  NOT NULL,
    content         TEXT                  NOT NULL,
    author_id  UUID                     NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX IF NOT EXISTS posts_author_id_index
    ON public.posts (author_id);

