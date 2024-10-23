BEGIN;

-- Users of the server
CREATE TABLE users (
    uuid UUID  -- Universal unique identifier
        PRIMARY KEY               -- This is how the table will be searched
        DEFAULT gen_random_uuid() -- If a uuid is not provided, generate one
    ,
    user_name TEXT -- The username
        UNIQUE                          -- Usernames are unique
        NOT NULL                        -- All users have a username
        CONSTRAINT user_name_has_no_whitespace CHECK (
                (user_name NOT LIKE '% %')  -- No whitespaces
            AND
                (user_name NOT LIKE E'%\t%')  -- No whitespaces
            AND
                (user_name NOT LIKE E'%\n%')  -- No whitespaces
        )
        CONSTRAINT user_name_is_not_empty CHECK
            (LENGTH(user_name) > 0) -- No empty usernames
    ,
    password TEXT  -- Hash of the password
        NOT NULL
    ,
    created TIMESTAMP WITH TIME ZONE -- Time of user creation
        NOT NULL                             -- Creation time is always populated
        DEFAULT current_timestamp            -- Default to the timestamp of user creation
        CONSTRAINT created_is_in_the_past CHECK 
            (created <= current_timestamp) -- Creation time is in the past
    ,
    last_access TIMESTAMP WITH TIME ZONE -- Time of last user access
        NOT NULL                                 -- Creation time is the first access
        DEFAULT current_timestamp                -- Default to the timestamp of user creation
        CONSTRAINT last_access_is_after_created CHECK 
            (last_access >= created) -- Cannot have last access be before creation
        CONSTRAINT last_access_is_in_the_past CHECK
            (last_access <= current_timestamp) -- Last access is in the past
);

-- Opened sessions
CREATE TABLE sessions (
    uuid UUID  -- Universal unique identifier
        PRIMARY KEY               -- This is how the table will be searched
        DEFAULT gen_random_uuid() -- If a uuid is not provided, generate one
    ,
    session_name TEXT -- Name of this session
        NOT NULL -- All sessions have a name
        CONSTRAINT session_name_has_no_margin_whitespace CHECK (
                (session_name NOT LIKE ' %')  -- No whitespaces
            AND
                (session_name NOT LIKE E'\t%')  -- No whitespaces
            AND
                (session_name NOT LIKE '% ')  -- No whitespaces
            AND
                (session_name NOT LIKE E'%\t')  -- No whitespaces
        )
        CONSTRAINT session_name_has_no_newlines CHECK
            (session_name NOT LIKE E'%\n%')  -- No newlines
        CONSTRAINT is_not_empty CHECK
            (LENGTH(session_name) > 0) -- No empty session names
    ,
    descr TEXT, -- Session description
    created TIMESTAMP WITH TIME ZONE -- Time of session creation
        NOT NULL                  -- Creation time is always populated
        DEFAULT current_timestamp -- Default to the timestamp of session creation
        CONSTRAINT created_is_in_the_past CHECK 
            (created <= current_timestamp) -- Creation time is in the past
    ,
    session_image BYTEA -- Session image: the serialized version of the session
        NULL -- If null, the session was created, but no data inserted
);

-- Users-session relationship

CREATE TYPE user_roles AS ENUM ('admin', 'player', 'observer');

CREATE TABLE sessions_users (
    session_uuid UUID -- ID of the session
        NOT NULL                  -- The session is always set in the relationship
        REFERENCES sessions(uuid) -- Always a valid session id
        ON UPDATE CASCADE         -- On session id change, update the relationship too
        ON DELETE CASCADE         -- On session delete, delete the relationship too
    ,
    user_uuid UUID -- ID of the user
        NOT NULL               -- The user is always set in the relationship
        REFERENCES users(uuid) -- Always a valid user id
        ON UPDATE CASCADE      -- On user id change, update the relationship too
        ON DELETE CASCADE      -- On user delete, delete the relationship too
    ,
    UNIQUE (session_uuid, user_uuid), -- Ensure uniqueness of the relationship
    user_role user_roles -- The user role for this session
        NOT NULL -- Every user has a role
    ,
    added TIMESTAMP WITH TIME ZONE -- When this user was added to this session
        NOT NULL                  -- Added time is always populated
        DEFAULT current_timestamp -- Default to the timestamp of relationship creation
        CONSTRAINT added_is_in_the_past CHECK 
            (added <= current_timestamp) -- Added time is in the past
    ,
    last_access TIMESTAMP WITH TIME ZONE -- When this user entered this session the last time
        NULL         -- The user might be added, but never accessed
        DEFAULT NULL -- At creation the user never entered
        CONSTRAINT last_access_is_in_the_past CHECK 
            (last_access <= current_timestamp) -- The last access must be in the past
        CONSTRAINT last_access_is_after_added CHECK 
            (last_access >= added) -- The last access must be after the added time
);

COMMIT;