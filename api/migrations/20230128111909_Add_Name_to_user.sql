-- The default is need to sattisfy the NOT NULL constraint,
-- but we don't want it so we instantly drop it
alter table users add column name text default '' not null ;
alter table users alter column name drop default;