# aiss

sequenceDiagram
    participant Client1
    participant Server
    participant Client2
    
    Client1->>Server: Register (TCP+UDP capabilities)
    Client2->>Server: Register (TCP+UDP capabilities)
    Server->>Client1: Send Client2 endpoints
    Server->>Client2: Send Client1 endpoints
    par TCP Connection
        Client1-->>Client2: TCP SYN
        Client2-->>Client1: TCP SYN
    and UDP Connection
        Client1-->>Client2: UDP Probe
        Client2-->>Client1: UDP Probe
    end
