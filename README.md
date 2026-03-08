# Quantum Core - Sistema Distribuido

## 1.Acerca

Este proyecto consiste en el desarrollo de un **sistema distribuido** haciendo
uso de tecnologías tales como **Docker**, **Rust**, **Kubernetes**,
**Wireguard**, y **QEMU** o **WSL2**, implementando una topología
**hub-and-spoke** y desarrollando un algoritmo distribuido en Rust.

## El sistema utiliza:

- WSL (Windows Subsystem for Linux) para ejecutar Linux en Windows
- WireGuard para crear una red privada segura
- Docker para contenerizar la aplicación
- Kubernetes para distribuir las tareas entre nodos
- Rust para ejecutar el algoritmo de Mandelbrot de forma eficiente
- El objetivo es distribuir el cálculo de la imagen entre varias máquinas, reduciendo el tiempo de procesamiento.

## 2.Arquitectura del Sistema.

El sistema sigue una arquitectura Master–Worker.

Componentes principales:

- Nodo maestro
  - coordina el cluster Kubernetes
  - distribuye tareas

- Nodos trabajadores
  - ejecutan contenedores con el programa en Rust
  - calculan una sección del Mandelbrot

- VPN WireGuard
  - conecta todas las computadoras en una red privada

- Flujo del sistema:
  - Las computadoras se conectan a la VPN
  - Se crea el cluster Kubernetes
  - Se ejecuta un job distribuido
  - Cada nodo calcula parte de la imagen
  - Los resultados se combinan

## 3.Topologia WireGuard
Se utilizó una topología estrella con WireGuard.
| Nodo | Dirección VPN | Rol    |
| ---- | ------------- | ------ |
| PC1  | 10.0.0.2      | Worker |
| PC2  | 10.0.0.3      | Worker |
| PC3  | 10.0.0.4      | Worker |
| PC4  | 10.0.0.5      | Worker |
| PC5  | 10.0.0.6      | Master |

## 4.Requisitos

Software necesario:

- Windows 10/11, Linux
- WSL2
- Ubuntu
- WireGuard
- Docker
- Kubernetes (k3s o kubeadm)
- Rust

Instalación básica:
- sudo apt update
- sudo apt install docker.io
- sudo apt install wireguard
- curl https://sh.rustup.rs -sSf | sh

## 5.Configuración de la VPN
Generar claves:  
wg genkey | tee privatekey | wg pubkey > publickey  
con esa configuracion creara una clave publica y privada y con esas mismas se creara las claves publicas de los clientes

Ejemplo de configuración del servidor:  
[Interface]  
Address = 10.0.0.1/24  
PrivateKey = SERVER_PRIVATE_KEY  
ListenPort = 51820  

[Peer]  
PublicKey = CLIENT_PUBLIC_KEY  
AllowedIPs = 10.0.0.2/32  

Iniciar WireGuard:  
sudo wg-quick up wg0  

Verificar conexión:  
ping 10.0.0.2  

## 6.Contenerización con Docker
El programa en Rust se empaqueta dentro de un contenedor.  

Ejemplo de Dockerfile:  
