-- Add migration script here
-- Add migration script here

-- ============================================================
-- RBAC: users, roles, permissions, sessions (UUID-based)
-- ============================================================

create extension if not exists pgcrypto; -- для gen_random_uuid()

create table users (
    id uuid primary key default gen_random_uuid(),
    email text unique not null,
    username text unique not null,
    password_hash text not null,
    is_active boolean not null default true,
    created_at timestamptz not null default now()
);

create table roles (
    id uuid primary key default gen_random_uuid(),
    code text unique not null,
    name_kz text not null,
    name_ru text not null,
    description_kz text,
    description_ru text
);

create table permissions (
    id uuid primary key default gen_random_uuid(),
    code text unique not null,
    name_kz text not null,
    name_ru text not null,
    description_kz text,
    description_ru text
);

create table user_roles (
    user_id uuid not null references users(id) on delete cascade,
    role_id uuid not null references roles(id) on delete cascade,
    primary key (user_id, role_id)
);

create table role_permissions (
    role_id uuid not null references roles(id) on delete cascade,
    permission_id uuid not null references permissions(id) on delete cascade,
    primary key (role_id, permission_id)
);

create table sessions (
    id uuid primary key default gen_random_uuid(),
    user_id uuid not null references users(id) on delete cascade,
    refresh_token_hash text not null,
    user_agent text,
    ip_address text,
    expires_at timestamptz not null,
    revoked boolean not null default false,
    created_at timestamptz not null default now()
);

create index idx_sessions_user_id on sessions(user_id);
create index idx_user_roles_user_id on user_roles(user_id);
create index idx_role_permissions_role_id on role_permissions(role_id);

-- ============================================================
-- БЕНТ: submissions, inspectors, categories, departments, etc.
-- ============================================================

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

create table departments (
    id bigint generated always as identity primary key,
    name_ru text not null,
    name_kz text,
    is_active boolean not null default true
);

create table categories (
    id bigint generated always as identity primary key,
    name_ru text not null,
    name_kz text,
    is_active boolean not null default true
);

create table inspectors (
    id bigint generated always as identity primary key,
    tabel_number bigint not null unique,
    name_ru text not null,
    name_kz text,
    is_active boolean not null default true,
    created_at timestamptz not null default now()
);

create table one_time_tokens (
    token text primary key,
    used boolean not null default false,
    telegram_user_id bigint not null,
    expires_at timestamptz not null,
    created_at timestamptz not null default now()
);

create table submissions (
    id bigint generated always as identity primary key,
    type submission_type not null,
    department_id bigint references departments(id),
    category_id bigint references categories(id),
    text text not null,
    file_url text,
    status submission_status not null default 'NEW',
    reject_reason text,
    channel text,
    telegram_user_id bigint,
    created_by_inspector bigint references inspectors(id),
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table submission_files (
    id bigint generated always as identity primary key,
    submission_id bigint not null references submissions(id) on delete cascade,
    file_url text not null,
    uploaded_at timestamptz not null default now()
);

create table submission_status_history (
    id bigint generated always as identity primary key,
    submission_id bigint not null references submissions(id) on delete cascade,
    old_status submission_status,
    new_status submission_status not null,
    changed_by uuid references users(id),
    comment text,
    changed_at timestamptz not null default now()
);

create table escalations (
    id bigint generated always as identity primary key,
    submission_id bigint not null references submissions(id) on delete cascade,
    level integer not null,
    sent_at timestamptz not null default now()
);

-- ============================================================
-- Seed: roles & permissions
-- ============================================================

insert into roles (id, code, name_kz, name_ru) values
  ('11111111-1111-1111-1111-111111111111', 'admin', 'Әкімші', 'Администратор'),
  ('22222222-2222-2222-2222-222222222222', 'user',  'Қолданушы', 'Пользователь');

insert into permissions (id, code, name_kz, name_ru) values
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'auth.me',         'Мені көру',       'Просмотр себя'),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'auth.logout',     'Шығу',            'Выход'),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'auth.logout_all', 'Барлығынан шығу', 'Выход отовсюду');

insert into role_permissions (role_id, permission_id) values
  ('22222222-2222-2222-2222-222222222222', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'),
  ('22222222-2222-2222-2222-222222222222', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'),
  ('22222222-2222-2222-2222-222222222222', 'cccccccc-cccc-cccc-cccc-cccccccccccc'),
  ('11111111-1111-1111-1111-111111111111', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'),
  ('11111111-1111-1111-1111-111111111111', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'),
  ('11111111-1111-1111-1111-111111111111', 'cccccccc-cccc-cccc-cccc-cccccccccccc');
