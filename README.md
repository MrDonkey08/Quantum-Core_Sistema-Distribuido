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

## 4 Instalación y Configuración

Para la instalación y configuración del sistema distribuido véase
[INSTALL](INSTALL.md).
