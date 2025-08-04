# File: signaling_server.py
# This script runs the signaling server. It's the central hub that allows the client and
# application server to discover each other's public IP and port.

import asyncio
import json

HOST = '0.0.0.0'
PORT = 8000
connections = {}
print("Starting signaling server on {}:{}".format(HOST, PORT))

async def handle_connection(reader, writer):
    """
    Handles a single connection to the signaling server.
    """
    addr = writer.get_extra_info('peername')
    print(f"New connection from {addr}")

    while True:
        try:
            data = await reader.read(4096)
            if not data:
                break
            
            message = json.loads(data.decode('utf-8'))
            print(f"Received message from {addr}: {message}")

            msg_type = message.get("type")

            if msg_type == "register":
                # An application server is registering its public IP and port.
                app_id = message.get("id")
                public_ip = message.get("public_ip")
                public_port = message.get("public_port")
                
                if app_id and public_ip and public_port:
                    connections[app_id] = {"public_ip": public_ip, "public_port": public_port, "writer": writer}
                    response = {"status": "success", "message": f"Registered with ID: {app_id}"}
                    writer.write(json.dumps(response).encode('utf-8'))
                    await writer.drain()
                    print(f"Application server '{app_id}' registered: {public_ip}:{public_port}")
                else:
                    response = {"status": "error", "message": "Invalid registration data."}
                    writer.write(json.dumps(response).encode('utf-8'))
                    await writer.drain()
            
            elif msg_type == "request":
                # A client is requesting the public IP and port of an application server.
                target_id = message.get("target_id")
                if target_id in connections:
                    target_info = connections[target_id]
                    # Send the target's info back to the requesting client.
                    response = {
                        "status": "success",
                        "target_ip": target_info["public_ip"],
                        "target_port": target_info["public_port"]
                    }
                    writer.write(json.dumps(response).encode('utf-8'))
                    await writer.drain()
                    print(f"Client from {addr} requested ID '{target_id}'. Sending back info.")
                    
                    # Also notify the application server that a client is trying to connect.
                    target_writer = target_info["writer"]
                    p2p_info = {
                        "type": "p2p_request",
                        "client_ip": addr[0],
                        "client_port": addr[1],
                        "message": "A client is initiating a P2P connection."
                    }
                    target_writer.write(json.dumps(p2p_info).encode('utf-8'))
                    await target_writer.drain()

                else:
                    response = {"status": "error", "message": "Application server ID not found."}
                    writer.write(json.dumps(response).encode('utf-8'))
                    await writer.drain()

        except (ConnectionError, json.JSONDecodeError) as e:
            print(f"Error with connection from {addr}: {e}")
            break

    print(f"Connection from {addr} closed.")
    # Clean up the connection dictionary if a server disconnects
    for app_id, info in list(connections.items()):
        if info["writer"] == writer:
            del connections[app_id]
            print(f"Application server '{app_id}' unregistered.")
            break

    writer.close()
    await writer.wait_closed()

async def main():
    """
    Main function to start the asyncio server.
    """
    server = await asyncio.start_server(handle_connection, HOST, PORT)
    async with server:
        await server.serve_forever()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Signaling server stopped.")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")
