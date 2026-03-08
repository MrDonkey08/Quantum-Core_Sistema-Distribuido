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
Para que todos los nodos puedan conectarse al Servidor VPN en Wireguard se realizó la instalación de Wireguard en cada máquina virtual de los distintos nodos. Utilizando el comando sudo apt install wireguard para realizar la instalación.​​
Se debe crear una carpeta para Wireguard con el comando sudo mkdir -p /etc/wireguard y crear el archivo de configuración con el comando sudo nano /etc/wireguard/wg0.conf, después se debe ajustar los permisos con sudo chmod 600 /etc/wireguard/wg0.conf. Para levantar la VPN se utiliza la instrucción sudo wg-quick up wg0 y verificar la conexión con wg-show.  

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

Dockerfile y Docker compose:  
para la configuracion del vaya al archivo readme agregado en la carpeta docker que se encuentra en este mismo repositorio.  
https://github.com/MrDonkey08/Quantum-Core_Sistema-Distribuido/blob/main/docker/README.md  
ejemplo de ejecucuion:  

FROM rust:1.93-trixie  

WORKDIR /app  

COPY Cargo.lock ./  
COPY Cargo.toml ./  

# PERFORMANCE: increase the rebuild speed because it loads all rust dependencies  
# See: https://www.youtube.com/watch?v=5Wfpzfniu4I  
RUN mkdir -p src \  
&& echo 'fn main() (}' > src/main.rs \  
&& cargo build -- release \  
# Remove dummy artifacts  
&& rm -rf src target/release/deps/mandelbrot*  

COPY ./src/ ./src/  
RUN cargo build -- release  

# Executes the pre-compiled rust release binary, just like "cargo run -- release"  
CMD ["./target/release/mandelbrot"]  

Dentro de un Sistema Distribuido es necesario la implementación de contenedores. Se establece un Dockerfile base para la creación de los contenedores. El Dockerfile utiliza la imagen oficial de Rust versión 1.93 y está basado en Linux Debian.​
Para gestionar los contenedores se utilizó el siguiente archivo de configuración docker-compose.yml  
​
## 7.Despliegue con Kubernetes  

Se utiliza Kubernetes para ejecutar múltiples instancias del programa.  
Ejemplo de job distribuido:  
  apiVersion: batch/v1  
  kind: Job  
  metadata:  
    name: mandelbrot-job  
  spec:  
    parallelism: 5  
    template:  
      spec:  
        containers:  
        - name: mandelbrot  
          image: mandelbrot:latest  
        restartPolicy: Never  

Ejecutar: kubectl apply -f mandelbrot-job.yaml  

Ver pods: kubectl get pods  

## 8. Algoritmo de Mandelbrot en Rust  

## 9. Ejecución del Sistema  
Pasos:  
- Iniciar la VPN: sudo wg-quick up wg0  
- Verificar nodos del cluster: kubectl get nodes
- Ejecutar el cálculo distribuido: kubectl apply -f mandelbrot-job.yaml
- Ver estado de pods: kubectl get pods    



