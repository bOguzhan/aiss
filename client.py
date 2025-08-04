# File: client.py
# This script runs the client. It connects to the signaling server to get the
# application server's public address and then initiates a P2P connection.

import socket
import json
import time

# Configuration
SIGNALING_SERVER_HOST = 'your-signaling-server-ip' # <--- REPLACE WITH YOUR IP
SIGNALING_SERVER_PORT = 8000
APP_SERVER_ID = 'your-app-server-id' # <--- REPLACE WITH THE TARGET ID


def get_app_server_info():
    """
    Connects to the signaling server and requests the public IP and port
    of the target application server.
    """
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((SIGNALING_SERVER_HOST, SIGNALING_SERVER_PORT))
            
            message = {
                "type": "request",
                "target_id": APP_SERVER_ID
            }
            
            sock.sendall(json.dumps(message).encode('utf-8'))
            
            response_data = sock.recv(1024)
            response = json.loads(response_data.decode('utf-8'))
            
            if response.get("status") == "success":
                return response.get("target_ip"), response.get("target_port")
            else:
                print(f"Error from signaling server: {response.get('message')}")
                return None, None
    except ConnectionError as e:
        print(f"Failed to connect to signaling server: {e}")
        return None, None

def punch_hole(sock, target_addr):
    """
    Sends a few packets to the target's public address to open a NAT hole.
    """
    print(f"Sending hole-punch packets to {target_addr}...")
    for _ in range(5):
        try:
            sock.sendto(b"HOLE_PUNCH", target_addr)
            time.sleep(0.5)
        except Exception as e:
            print(f"Failed to send hole-punch packet: {e}")

def handle_p2p_communication(sock, target_addr):
    """
    Handles the P2P communication after the hole has been punched.
    """
    print(f"P2P session established with {target_addr}. You can now send messages.")
    while True:
        try:
            message = input("Enter message to application server ('exit' to quit): ")
            if message.lower() == 'exit':
                break
            sock.sendto(message.encode('utf-8'), target_addr)
            
            sock.settimeout(5)
            data, addr = sock.recvfrom(1024)
            print(f"Received from app server: {data.decode('utf-8')}")

        except socket.timeout:
            print("Timeout waiting for response. Continuing...")
            continue
        except Exception as e:
            print(f"P2P communication error: {e}")
            break

    print("P2P session closed.")


if __name__ == "__main__":
    if APP_SERVER_ID == 'your-app-server-id':
        print("Please replace 'your-app-server-id' in the script with the actual ID of the application server.")
        exit()

    if SIGNALING_SERVER_HOST == 'your-signaling-server-ip':
        print("Please replace 'your-signaling-server-ip' in the script with the actual IP address of your signaling server.")
        exit()

    print(f"Client is attempting to connect to application server with ID: {APP_SERVER_ID}")
    
    target_ip, target_port = get_app_server_info()
    
    if target_ip and target_port:
        print(f"Received target info from signaling server: {target_ip}:{target_port}")
        
        try:
            # Create a UDP socket for P2P communication
            p2p_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            
            # Start the hole punching process
            target_addr = (target_ip, target_port)
            punch_hole(p2p_socket, target_addr)

            # Now, try to receive the confirmation from the server
            p2p_socket.settimeout(5)
            print("Waiting for hole-punch acknowledgment from application server...")
            try:
                data, addr = p2p_socket.recvfrom(1024)
                if data.decode('utf-8') == "HOLE_PUNCH_ACK":
                    print(f"Hole-punch acknowledgment received from {addr}. P2P connection established!")
                    handle_p2p_communication(p2p_socket, target_addr)
                else:
                    print(f"Received unexpected response: {data.decode('utf-8')}")

            except socket.timeout:
                print("Hole-punch acknowledgment timed out. P2P connection failed.")
            
            p2p_socket.close()

        except Exception as e:
            print(f"An error occurred during P2P setup: {e}")

    else:
        print("Failed to get application server information. Exiting.")
