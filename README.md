# moly
UDP hole punch tool using a relay server

## Usage
- Start server:
  ```
  moly server PORT MAX_HOSTS
  ```
  Where PORT is the port to which moly will bind and MAX_HOSTS is the maximum number of host connections to allow
- Connect as host:
  ```
  moly host PORT NAME SERVER
  ```
  Where PORT is the port to which the service you want to make accessible over moly is bound and NAME is the name with
  which moly will register on the server reachable with the address given as SERVER
- Connect as client:
  ```
  moly client PORT NAME SERVER
  ```
  Where PORT is the port to which local programs can access the connection over moly, NAME is the name of the host which
  is to be reached and SERVER is the address of the relay server.
