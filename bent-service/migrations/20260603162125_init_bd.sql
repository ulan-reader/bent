-- Add migration script here

create type submission_type as enum (
    'EMPLOYEE',
    'INSPECTOR'
);

create type submission_status as enum (
    'NEW',
    'IN_PROGRESS',
    'REJECTED',
    'COMPLETED'
);


create table submissions (
    id bigint generated always as identity primary key,

    type submission_type not null,

    department_id bigint,
    category_id bigint,

    text text not null,

    file_url text,

    status submission_status not null default 'NEW',

    reject_reason text,

    channel text,

    telegram_user_id bigint,

    created_by_inspector bigint,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table inspectors (
    id bigint generated always as identity primary key,

    tabel_number bigint not null unique,

    name_ru text not null,
    name_kz text,

    is_active boolean not null default true,

    created_at timestamptz not null default now()
);

create table users (
    id bigint generated always as identity primary key,

    email text not null unique,
    password_hash text not null,

    first_name text not null,
    second_name text not null,
    middle_name text,

    is_active boolean not null default true,

    created_at timestamptz not null default now()
);

create table roles (
    id bigint generated always as identity primary key,

    name text not null,
    code text not null unique
);

create table user_roles (
    user_id bigint not null references users(id) on delete cascade,
    role_id bigint not null references roles(id) on delete cascade,

    primary key(user_id, role_id)
);

create table one_time_tokens (
    token text primary key,

    used boolean not null default false,

    telegram_user_id bigint not null,

    expires_at timestamptz not null,

    created_at timestamptz not null default now()
);

create table escalations (
    id bigint generated always as identity primary key,

    submission_id bigint not null
        references submissions(id)
        on delete cascade,

    level integer not null,

    sent_at timestamptz not null default now()
);


create table submission_status_history (
    id bigint generated always as identity primary key,

    submission_id bigint not null
        references submissions(id)
        on delete cascade,

    old_status submission_status,

    new_status submission_status not null,

    changed_by bigint
        references users(id),

    comment text,

    changed_at timestamptz not null default now()
);

create table categories (
    id bigint generated always as identity primary key,

    name_ru text not null,
    name_kz text,

    is_active boolean not null default true
);

create table departments (
    id bigint generated always as identity primary key,

    name_ru text not null,
    name_kz text,

    is_active boolean not null default true
);

create table submission_files (
    id bigint generated always as identity primary key,

    submission_id bigint not null
        references submissions(id)
        on delete cascade,

    file_url text not null,

    uploaded_at timestamptz not null default now()
);
