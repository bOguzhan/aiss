# File: app_server.py
# This script runs the application server. It registers with the signaling server
# and waits for a client to initiate a P2P connection.

import socket
import json
import time
import uuid

# Configuration
SIGNALING_SERVER_HOST = 'your-signaling-server-ip' # <--- REPLACE WITH YOUR IP
SIGNALING_SERVER_PORT = 8000
APP_SERVER_ID = str(uuid.uuid4()) # A unique ID for the application server
APP_SERVER_PORT = 9000
STUN_SERVER = 'stun.l.google.com'
STUN_PORT = 19302

def get_public_ip_and_port():
    """
    Uses a public STUN server to discover the public IP and port of the machine.
    This is a basic implementation of a STUN client.
    """
    print("Attempting to get public IP and port via STUN...")
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.settimeout(5)
        
        # STUN Binding Request message format
        stun_message = b'\x00\x01\x00\x00\x21\x12\xa4\x42'
        sock.sendto(stun_message, (STUN_SERVER, STUN_PORT))
        
        data, addr = sock.recvfrom(1024)
        
        # Parse STUN Binding Response
        # The MAPPED-ADDRESS attribute starts at byte 24
        ip_bytes = data[24:28]
        port_bytes = data[22:24]
        
        public_ip = '.'.join(map(str, ip_bytes))
        public_port = int.from_bytes(port_bytes, 'big')
        
        return public_ip, public_port

    except socket.timeout:
        print("STUN request timed out. Please check your network connection.")
    except Exception as e:
        print(f"An error occurred during STUN request: {e}")
    finally:
        sock.close()
    
    return None, None


def register_with_signaling_server(public_ip, public_port):
    """
    Connects to the signaling server and registers the application server's ID and public address.
    """
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((SIGNALING_SERVER_HOST, SIGNALING_SERVER_PORT))
            
            message = {
                "type": "register",
                "id": APP_SERVER_ID,
                "public_ip": public_ip,
                "public_port": public_port
            }
            
            sock.sendall(json.dumps(message).encode('utf-8'))
            
            response = sock.recv(1024)
            print(f"Signaling server response: {response.decode('utf-8')}")

            return sock # Return the socket to keep the connection alive for receiving requests
    except ConnectionError as e:
        print(f"Failed to connect to signaling server: {e}")
        return None


def listen_for_p2p_request(signaling_socket, public_ip, public_port):
    """
    Keeps the signaling connection alive to receive P2P connection requests from clients.
    Also listens on a UDP socket for the initial hole-punching packets.
    """
    udp_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    udp_socket.bind(('0.0.0.0', APP_SERVER_PORT))
    udp_socket.settimeout(1) # Small timeout to allow for periodic checks

    print(f"Application server '{APP_SERVER_ID}' is registered. Listening for requests on UDP port {public_port}...")
    
    while True:
        try:
            # Check for messages from the signaling server
            signaling_socket.settimeout(0.5)
            signaling_data = signaling_socket.recv(4096)
            if signaling_data:
                request = json.loads(signaling_data.decode('utf-8'))
                if request.get("type") == "p2p_request":
                    client_ip = request.get("client_ip")
                    client_port = request.get("client_port")
                    print(f"Received P2P request from signaling server for client: {client_ip}:{client_port}")
                    
                    # Start UDP hole punching
                    punch_hole(udp_socket, (client_ip, client_port))

            # Listen for initial UDP packet from client (the "hole punch")
            try:
                data, addr = udp_socket.recvfrom(1024)
                if data.decode('utf-8') == "HOLE_PUNCH":
                    print(f"Received hole-punch packet from {addr}. Connection established!")
                    
                    # Send a confirmation back to the client
                    udp_socket.sendto(b"HOLE_PUNCH_ACK", addr)

                    # Now you can start the actual P2P communication
                    handle_p2p_communication(udp_socket, addr)
                    break # Exit the loop after successful connection

            except socket.timeout:
                pass # Continue listening

        except (ConnectionError, socket.timeout) as e:
            print(f"Signaling connection lost or timeout: {e}")
            break
        except Exception as e:
            print(f"An unexpected error occurred: {e}")
            break

    udp_socket.close()
    signaling_socket.close()

def punch_hole(udp_socket, client_addr):
    """
    Sends a few packets to the client's public address to open a NAT hole.
    """
    print(f"Sending hole-punch packets to {client_addr}...")
    for _ in range(5):
        try:
            udp_socket.sendto(b"HOLE_PUNCH", client_addr)
            time.sleep(0.5)
        except Exception as e:
            print(f"Failed to send hole-punch packet: {e}")

def handle_p2p_communication(sock, client_addr):
    """
    Handles the P2P communication after the hole has been punched.
    """
    print(f"P2P session established with {client_addr}. You can now send messages.")
    while True:
        try:
            message = input("Enter message to client ('exit' to quit): ")
            if message.lower() == 'exit':
                break
            sock.sendto(message.encode('utf-8'), client_addr)
            
            data, addr = sock.recvfrom(1024)
            print(f"Received from client: {data.decode('utf-8')}")

        except socket.timeout:
            continue
        except Exception as e:
            print(f"P2P communication error: {e}")
            break

    print("P2P session closed.")


if __name__ == "__main__":
    print(f"Application Server ID is: {APP_SERVER_ID}")
    public_ip, public_port = get_public_ip_and_port()
    
    if public_ip and public_port:
        print(f"Public IP: {public_ip}, Public Port: {public_port}")
        
        signaling_socket = register_with_signaling_server(public_ip, public_port)
        if signaling_socket:
            listen_for_p2p_request(signaling_socket, public_ip, public_port)
    else:
        print("Failed to get public IP/port. Cannot register with signaling server.")
