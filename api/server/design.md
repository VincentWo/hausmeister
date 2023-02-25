# General Design Thoughts
This document tries to be both: an overview of the current state of the app and
a plan how to add extended functionality

## DB or Redis?

## Sessions
Sessions are represented by a single v4 uuid, this makes them completely opaque
and uneditable by any client whatsoever. Currently the season is only set using
the authorization header with "Bearer \[session_id\]" but allowing cookies
might make sense in the future since setting these HTTP-Only would add another
layer of protection from XSS-Attacks.

### Extraction
The session middleware adds a SessionId(Uuid) to every requests if an
