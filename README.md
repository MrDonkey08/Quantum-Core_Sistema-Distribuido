# Quantum Core: Sistema Distribuido para la Generación del Conjunto de Mandelbrot

## 1. Acerca

Este proyecto consiste en el desarrollo de un **sistema distribuido** haciendo
uso de tecnologías tales como **Docker**, **Rust**, **Kubernetes**,
**WireGuard**, y **QEMU** o **WSL2**, implementando una topología
**hub-and-spoke** y desarrollando un algoritmo distribuido en Rust.

## 2. Requisitos de Software

A continuación se muestra el software requerido por cada uno de los nodos
(hosts) del sistema distribuido:

> [!TIP]
>
> Puede que versiones anteriores del software funcionen. Las versiones
> especificadas son aquellas que fueron probadas.

| Software              | Propósito                                                                      |             Versión | Forzozamente requerido |
| --------------------- | ------------------------------------------------------------------------------ | ------------------: | ---------------------- |
| **WireGuard**         | Para establecer una conexión segura entre los nodos a través de Internet       |      1.0.20210914 + | Sí                     |
| **Docker**            | Para la creación de los contenedores que ejecutará cada host                   |            29.3.0 + | Sí                     |
| **K3s**               | Para la orquestación de los pods (contenedores)                                |       1.34.5+k3s1 + | Sí                     |
| **iptables**          | Para el filtrado de paquetes IPv4/IPv6                                         |            1.8.11 + | No                     |
| **iperf3**            | Para pruebas de rendimiento de throughput entre el servidor y los clientes VPN | 3.18 (cJSON 1.7.15) | No                     |
| **ip** o **ifconfig** | Para visualizar las interfaces de red                                          |                 N/A | No                     |
| **ping**              | Para probar la conectividad entre los nodos                                    |                 N/A | No                     |

> [!NOTE]
>
> Cabe destacar que, como entorno de prueba, usamos como nodos máquinas
> virtuales, específicamente _**WSL2 con Ubuntu** para los usuarios de **Windows
> 10/11**_ y _**QEMU con Debian** para los usuarios de **Linux**_ (en nuestro
> caso Arch Linux).
>
> Para más información del entorno de los nodos (hosts) véase
> [Issue 2](https://github.com/MrDonkey08/Quantum-Core_Sistema-Distribuido/issues/2)

## 3. Arquitectura del Sistema Distribuido (VPN + K3s)

Como arquitectura del sistema distribuido utilizamos una topología
**hub-and-spoke**, dónde el **hub es el servidor VPN** y los **hubs** son los
clientes VPN.

Para nuestra arquitectura, utilizamos una configuración que permitiese a
cualquier nodo ser el **servidor K3s**, ofreciéndonos flexibilidad al momento de
trabajar con K3s.

### Roles de los Nodos

Los roles que establecimos para los nodos son:

| Nodo | Rol VPN  | IP VPN | Rol K3s       |
| ---- | -------- | ------ | ------------- |
| 1    | Servidor | .1     | N/A           |
| 2    | Cliente  | .2     | \<none\>      |
| 3    | Cliente  | .4     | control-plane |
| 4    | Cliente  | .5     | \<none\>      |
| 5    | Cliente  | .6     | \<none\>      |
| 6    | Cliente  | .8     | \<none\>      |

Dónde:

- La red de la VPN es la `10.5.5.0/24`

- El **control-plane** es el **servidor K3s**, el cuál se encarga de coordinar
  el clúster, distribuyendo los 21 pods (1 _coordinators_ y 20 _workers_) nodos
  a sí mismo y al resto de los nodos.

- Los nodos **\<none\>** son los **agentes (agents) K3S**, los cuáles se
  conectarán al _control-plane_ y recibirán los nodos.

## 4. Instalación y Configuración

Para la instalación y configuración del sistema distribuido véase
[INSTALL](INSTALL.md).

## 5. Uso

Una vez instalado y configurado cada uno de los nodos, servidores y toda la
infraestructura necesaria para el sistema distribuido, podemos proceder a hacer
uso del algoritmo de Mandelbrot distribuido.

### 5.1. Nos Conectamos al Servidor VPN

```bash
HOST_IP=$(grep nameserver /etc/resolv.conf | awk '{print $2}')
echo "$HOST_IP"
sudo ip route add 10.5.5.0/24 via "$HOST_IP"

sudo wg-quick down wg0 2>/dev/null
sudo wg-quick up wg0
sudo wg show wg0 # Visualizar la interfaz activa wg0
ping 10.5.5.1   # Probar la conectividad con el servidor VPN

sudo ip route show | grep 10.5.5
sudo ip route del 10.5.5.0/24 2>/dev/null
sudo ip route add 10.5.5.0/24 dev wg0
sudo ip route show | grep 10.5.5

# Configuración de flannel
sudo iptables -I INPUT -i flannel.1 -j ACCEPT
sudo iptables -I FORWARD -i flannel.1 -j ACCEPT
sudo iptables -I OUTPUT -o flannel.1 -j ACCEPT

# Configuración de cni0
sudo iptables -I INPUT -i cni0 -j ACCEPT
sudo iptables -I FORWARD -i cni0 -j ACCEPT
```

### 5.2. Levantamos el Servidor y los Agents K3s

1. Primeramente, el nodo con _servidor K3s_ ejecuta los siguientes comandos para
   levantar el servidor:

   ```bash
   sudo systemctl start k3s # En caso de no estar habilitado y activo
   # Nos aseguramos de que se encuentre "Active (running)"
   sudo systemctl status k3s
   ```

2. Una vez levantado el servidor, los nodos _agent K3s_ ejecutan los siguientes
   comandos:

   ```bash
   sudo systemctl start k3s-agent # En caso de no estar habilitado y activo
   # Nos aseguramos de que se encuentre "Active (running)"
   sudo systemctl status k3s-agent
   ```

3. Una vez que todos los nodos agents se encuentren activos, el servidor puede
   ejecutar cualquiera de los siguientes comandos para visualizar el estado de
   los nodos:

   ```bash
   sudo kubectl get nodes
   sudo kubectl get nodes -o wide # Muestra más datos
   ```

### 5.3. Creamos y Exportamos Nuestra Imagen Docker a K3s

Procedemos a crear y exportar la imagen Docker a K3s tanto en el _servidor K3s_
como en cada uno de los _agents K3s_ siguiendo los pasos a continuación:

1. Nos posicionamos en la carpeta raíz del repositorio.

2. Creamos la imagen Docker a partir de nuestro
   [Dockerfile](./docker/Dockerfile):

   ```bash
   docker build -f docker/Dockerfile -t quantum-core:1.0 ./rust
   ```

   > [!TIP]
   >
   > En caso de error por no poder descargar automáticamente las imágenes base
   > del [Dockerfile](./docker/Dockerfile), procedemos a descargarlas
   > manualmente, por ejemplo:
   >
   > ```bash
   > # Imagen Rust base usada para compilar el ejecutable
   > docker pull rust:1.93-slim-trixie
   > docker pull debian:trixie-slim # Imagen base final
   > ```

3. Exportamos la imagen Docker a K3s:

   ```bash
   docker save quantum-core:1.0 | sudo k3s ctr images import -
   ```

### 5.4. Creamos los _pods_ desde el Servidor

1. Creamos los manifestos:

   ```bash
   # Eliminamos los manifestos existentes (también elimina los pods existentes)
   sudo kubectl delete -f k8s/

   # Aplicamos los manifestos en orden
   sudo kubectl apply -f ./k8s/coordinator-service.yml
   sudo kubectl apply -f ./k8s/coordinator-deployment.yaml
   sudo kubectl apply -f ./k8s/worker-headless-service.yaml
   sudo kubectl apply -f ./k8s/worker-statefulset.yaml
   ```

2. Nos aseguramos de que todos los pods se hayan creado exitosamente:

   ```bash
   sudo kubectl get pods
   sudo kubectl get pods -o wide # Muestra más datos
   ```

### 5.5. Visualizamos los Logs y la Imagen Generada desde el Servidor

1. Para visualizar los logs del _coordinator_ ejecutamos cualquiera de los
   siguientes comandos:

   ```bash
   sudo kubectl logs <nombre-del-pod-coordinator>
   sudo kubectl logs deployment/coordinator
   ```

2. Para visualizar los logs de un _worker_ en específico ejecutamos:

   ```bash
   sudo kubectl logs <nombre-del-pod-worker>
   ```

3. Cuando el programa distribuido finalice, habrá generado la imagen del fractal
   del conjunto de Mandelbrot en la ruta `/tmp/mandelbrot/output/fractal.png`.

> [!TIP]
>
> Podemos visualizar la imagen desde la terminal haciendo uso de cualquiera de
> los siguientes comandos:
>
> ```bash
> viu /tmp/mandelbrot/output/fractal.png
> chafa /tmp/mandelbrot/output/fractal.png
> display /tmp/mandelbrot/output/fractal.png
> ```
