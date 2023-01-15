# golem

Server Management app

- Backend (Codename Gabit):
  - Uses Rust and Rocket
  - Handle getting info from servers
  - Expose a REST api for getting the info
- Info Gatherer (Codename Zekrom):
  - Uses Rust
  - Gets infomation from the running system
  - Exposes REST api so the backend can gather the system infomation
    - GET: /api/capabilities
      - Returns the capabilities of the system
    - GET: /api/system
      - Returns system info (CPU, RAM, OS Version)
    - GET: /api/docker
      - Returns docker infomation
- Flutter Frontend (Codename Flittle):
  - Uses Flutter
  - Use the REST api to build the frontend
