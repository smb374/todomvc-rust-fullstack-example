-- Your SQL goes here
create table task (
    id uuid not null unique,
    content text not null,
    completed boolean not null,
    editing boolean not null,
    primary key (id)
);
