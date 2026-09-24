# SQLite for local persistence

Users, Tasks, login challenges, and email logs are stored in one SQLite file. The connection string is a `sqlite://` URL. Schema changes go through SQLx migrations against that file.
