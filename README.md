# shorty-rs
An url shortener written in plain rust, 
tried to use the std lib as much as possible to learn http.
No http lib used.

Libs used for convinience:

- chrono for logging
- regex ... for regex 🙉
- rand for generating random urls
- rusqlite for database

Currently, you have to set the env-var SHORTY_BASE_URL to the
base url of this service, so the interpolated links of the "created"
page work. Otherwise, this service is hosting-url independent.

(Could also fix this by applying the prefix via ts like in the index page)

The database is stored in the same dir as the executable (see database.sqlite in this repo)
and contains two tables:

- `passwords`:
    
    only row `password`: valid passwords, so they can get changed.
    
- `urls`:
    - `short`: the shortened url suffix (reachable through "SHORTY_BASE_URL"/"short")
    - `long`: the long url (with http(s):// prefix!)
    - `ip_hash`: The u32 hashed ip to prevent url spamming
    - `created`: when this shorty was created
    - `last_redirect`: when this shorty was last used
    - `redirects`: how often this shorty was used
    
Current version is single-threaded.

#TODOS
- Fix unsafe Regex caching
- multithreading!!! (one listnener, than distribution)
- check all unwraps / expects and if they are safe.
- write more (doc)comments