# Quantum Core: Sistema Distribuido para la Generación del Conjunto de Mandelbrot

## 1. Acerca

Este proyecto consiste en el desarrollo de un **sistema distribuido** haciendo
uso de tecnologías tales como **Docker**, **Rust**, **Kubernetes**,
**WireGuard**, y **QEMU** o **WSL2**, implementando una topología
**hub-and-spoke** y desarrollando un algoritmo distribuido en Rust.

### El Sistema Utiliza

- WSL (Windows Subsystem for Linux) para ejecutar Linux en Windows
- WireGuard para crear una red privada segura
- Docker para contenedorizar la aplicación
- Kubernetes para distribuir las tareas entre nodos
- Rust para ejecutar el algoritmo de Mandelbrot de forma eficiente
- El objetivo es distribuir el cálculo de la imagen entre varias máquinas,
  reduciendo el tiempo de procesamiento.

## 2. Arquitectura del Sistema

El sistema sigue una arquitectura Master–Worker.

### Componentes principales

- Nodo maestro
  - Coordina el clúster Kubernetes
  - Distribuye tareas

- Nodos trabajadores
  - Ejecutan contenedores con el programa en Rust
  - Calculan una sección del Mandelbrot

- VPN WireGuard
  - Conecta todas las computadoras en una red privada

- Flujo del sistema:
  - Las computadoras se conectan a la VPN
  - Se crea el clúster Kubernetes
  - Se ejecuta un job distribuido
  - Cada nodo calcula parte de la imagen
  - Los resultados se combinan

## 3. Topología WireGuard

Se utilizó una **topología estrella (Hub-and-Spoke)** con WireGuard.

Todas las computadoras se conectan al nodo maestro, el cual funciona como
servidor VPN y punto central de comunicación.

Una vez conectados a la VPN, se crea un **clúster de Kubernetes** que permite
ejecutar tareas distribuidas entre los nodos.

Para el cálculo del conjunto de Mandelbrot se ejecutarán **20 contenedores
(pods)** en Kubernetes.

El **orquestador de Kubernetes** se encargará de distribuir automáticamente
estos contenedores entre las diferentes máquinas del clúster dependiendo de los
recursos disponibles.

| Nodo | Dirección VPN | Rol    |
| ---- | ------------- | ------ |
| PC1  | 10.0.0.2      | Worker |
| PC2  | 10.0.0.3      | Worker |
| PC3  | 10.0.0.4      | Worker |
| PC4  | 10.0.0.5      | Worker |
| PC5  | 10.0.0.6      | Master |

## 4. Requisitos

Para ejecutar el sistema distribuido se requiere contar con una distribución de
Linux ejecutándose en una máquina virtual.

Dependiendo del sistema operativo se utilizaron las siguientes tecnologías:

- **Windows 10 / Windows 11**
  - WSL2 (Windows Subsystem for Linux)

- **Linux**
  - QEMU para virtualización

Software necesario:

- Ubuntu 22.04 o superior
- WireGuard
- Docker
- Kubernetes (k3s o kubeadm)
- Rust
- Git

Instalación básica en Ubuntu:

```bash
sudo apt update
sudo apt install docker.io
sudo apt install wireguard
curl https://sh.rustup.rs -sSf | sh
```

## 5. Configuración de la VPN

Para que todos los nodos puedan conectarse al servidor VPN en WireGuard se
realizó la instalación de WireGuard en cada máquina virtual de los distintos
nodos, utilizando el comando `sudo apt install wireguard`.

Se debe crear una carpeta para WireGuard con el comando
`sudo mkdir -p /etc/wireguard` y crear el archivo de configuración con el
comando `sudo nano /etc/wireguard/wg0.conf`. Después se deben ajustar los
permisos con `sudo chmod 600 /etc/wireguard/wg0.conf`. Para levantar la VPN se
utiliza `sudo wg-quick up wg0` y para verificar la conexión, `wg show`.

### Generación de Llaves

```bash
wg genkey | tee privatekey | wg pubkey > publickey
```

Con esa configuración se crearán una clave pública y privada, y con esas mismas
se crearán las claves públicas de los clientes.

#### Ejemplo de Configuración del Servidor

Archivo `/etc/wireguard/wg0.conf` del servidor:

```text
[Interface]
Address = 10.0.0.1/24
PrivateKey = SERVER_PRIVATE_KEY
ListenPort = 51820

[Peer]
PublicKey = CLIENT_PUBLIC_KEY
AllowedIPs = 10.0.0.2/32
```

Iniciar WireGuard:

```bash
sudo wg-quick up wg0
```

Verificar conexión:

```bash
ping 10.0.0.1
```

## 6. Contenedorización con Docker

El programa en Rust se empaqueta dentro de un contenedor.

### Dockerfile

```dockerfile
FROM rust:1.93-trixie

WORKDIR /app

COPY Cargo.lock ./
COPY Cargo.toml ./

# PERFORMANCE: increase the rebuild speed because it loads all rust dependencies
# See: https://www.youtube.com/watch?v=5Wfpzfniu4I
RUN mkdir -p src \
&& echo 'fn main() {}' > src/main.rs \
&& cargo build --release \
# Remove dummy artifacts
&& rm -rf src target/release/deps/mandelbrot*

COPY ./src/ ./src/
RUN cargo build --release

# Executes the pre-compiled rust release binary, just like "cargo run --release"
CMD ["./target/release/mandelbrot"]
```

Dentro de un sistema distribuido es necesaria la implementación de contenedores.
Se establece un Dockerfile base para la creación de los contenedores. El
Dockerfile utiliza la imagen oficial de Rust versión 1.93 y está basado en
Debian Linux.

## 7. Despliegue con Kubernetes

Se utiliza Kubernetes para ejecutar múltiples instancias del programa. Ejemplo
de job distribuido:

```yaml
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
```

Ejecutar:

```bash
kubectl apply -f mandelbrot-job.yaml
```

Ver pods:

```bash
kubectl get pods
```

## 8. Algoritmo de Mandelbrot en Rust

Instalación de Rust

Instalar Rust utilizando rustup:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Crear un proyecto:

```bash
cargo new mandelbrot_worker
cd mandelbrot_worker
```

### Ejemplo de Código del Worker en Rust

Archivo `src/main.rs`:

```rust
use std::env;

fn mandelbrot(x: f64, y: f64, max_iter: u32) -> u32 {
    let mut zx = 0.0;
    let mut zy = 0.0;
    let mut iter = 0;

    while zx*zx + zy*zy <= 4.0 && iter < max_iter {
        let temp = zx*zx - zy*zy + x;
        zy = 2.0*zx*zy + y;
        zx = temp;
        iter += 1;
    }

    iter
}

fn main() {

    let args: Vec<String> = env::args().collect();

    let start: f64 = args[1].parse().unwrap();
    let end: f64 = args[2].parse().unwrap();

    let mut total = 0;

    for i in 0..1000 {

        let x = start + (end-start)*(i as f64)/1000.0;

        for j in 0..1000 {

            let y = -1.0 + 2.0*(j as f64)/1000.0;

            total += mandelbrot(x,y,1000);

        }
    }

    println!("Resultado parcial: {}", total);
}
```

### Crear Imagen Docker

Archivo `Dockerfile`:

```dockerfile
FROM rust:1.75

WORKDIR /app

COPY . .

RUN cargo build --release

CMD ["./target/release/mandelbrot_worker","-2.0","1.0"]
```

Construir imagen:

```bash
docker build -t mandelbrot-worker .
```

Subir imagen a DockerHub:

```bash
docker tag mandelbrot-worker usuario/mandelbrot-worker
docker push usuario/mandelbrot-worker
```

### Ejecución Distribuida con Kubernetes

Archivo `mandelbrot-job.yaml`:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: mandelbrot-job
spec:
  completions: 20
  parallelism: 5
  template:
    spec:
      containers:
        - name: mandelbrot
          image: usuario/mandelbrot-worker
      restartPolicy: Never
```

## 9. Ejecución del Sistema

```bash
# 1. Iniciar la VPN
sudo wg-quick up wg0
# 2. Verificar nodos del clúster
kubectl get nodes
# 3. Ejecutar el cálculo distribuido
kubectl apply -f mandelbrot-job.yaml
# 4. Ver estado de pods
kubectl get pods
```
