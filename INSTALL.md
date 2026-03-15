# Guía para la Instalación y Configuración

## Acerca

En este documento se describen los pasos necesarios para la instalación y
configuración de los nodos, específicamente para las distribuciones de Ubuntu y
Debian.

> [!NOTE]
>
> Para otras distribuciones, algunos de los pasos pueden cambiar, especialmente
> si no están basadas en Ubuntu o Debian.

## Actualización del Sistema

Se recomienda siempre actualizar primero el sistema. Para ello ejecutamos:

```bash
sudo apt update && sudo apt upgrade
```

Donde:

- `sudo apt update` actualiza los repositorios de paquetes
- `sudo apt upgrade` actualiza los paquetes del sistema

## Instalación y Configuración de WireGuard

### Instalación de WireGuard

```bash
sudo apt install wireguard
```

### Generación de Llaves

1. Creamos una carpeta temporal para generar los _key pairs_ y nos posicionamos
   en ella:

   ```bash
   mkdir /tmp/keys && cd /tmp/keys
   ```

2. Creamos la **llave privada**:

   ```bash
   wg genkey > peer_A.key
   ```

3. Generamos la **llave pública** a partir de la **llave privada**:

   ```bash
   wg pubkey < peer_A.key > peer_A.pub
   ```

### Archivo de Configuración de un Peer

1. Primero creamos la **llave pública y privada del _peer_**. Para ello véase la
   sección [Generación de Llaves](#generación-de-llaves).

2. Después creamos el archivo de configuración para el _peer_ en la ruta
   `/etc/wireguard/wg0.conf`, haciendo uso de los _key pairs_ generados del
   cliente, por ejemplo:

   ```text
   [Interface]
   PrivateKey = LLAVE_PRIVADA_SERVIDOR
   Address = 10.0.0.1/24
   DNS = 8.8.8.8 4.4.4.4
   PostUp = ip route del 10.5.5.0/24 2>/dev/null; ip route add 10.5.5.0/24 dev wg0
   PostDown = ip route del 10.5.5.0/24 2>/dev/null

   [Peer]
   PublicKey = LLAVE_PÚBLICA_PEER_A
   Endpoint = IP_PÚBLICA_SERVIDOR
   AllowedIPs = 0.0.0.0/0
   PersistentKeepalive = 25
   ```

### Archivo de Configuración del Servidor

1. Primero creamos la **llave pública y privada del servidor**. Para ello véase
   la sección [Generación de Llaves](#generación-de-llaves).

2. Asimismo creamos las llaves de cada _peer_ (de no haberlas creado
   previamente).

3. Después creamos el archivo de configuración para el servidor en la ruta
   `/etc/wireguard/wg0.conf`, haciendo uso de los _key pairs_ generados de cada
   cliente y del servidor, por ejemplo:

   ```text
   [Interface]
   Address = 10.0.0.1/24
   ListenPort = 51820
   PrivateKey = LLAVE_PRIVADA_SERVIDOR
   PostUp = sysctl -w net.ipv4.ip_forward=1

   [Peer]
   PublicKey = LLAVE_PÚBLICA_PEER_FOO
   AllowedIPs = 10.0.0.2/32

   [Peer]
   PublicKey = LLAVE_PÚBLICA_PEER_BAR
   AllowedIPs = 10.0.0.3/32
   ```

### Iniciar WireGuard y Probar Conexión

1. Para iniciar WireGuard, levantamos la interfaz `wg0` ejecutando:

   ```bash
   sudo wg-quick up wg0
   ```

2. Una vez iniciada la interfaz `wg0` tanto en el servidor como en el _peer_,
   podemos probar la conexión desde el _peer_ ejecutando:

   ```bash
   ping 10.0.0.1
   ```

## Instalación de Docker

```bash
sudo apt install docker
```

## Instalación de K3s

### Instalación del Servidor K3s

Para instalar k3s como servidor, ejecutamos el siguiente comando:

```bash
curl -sfL https://get.k3s.io | \
  sh \
    -s - server \
      --bind-address=<ip-del-nodo> \
      --advertise-address=<ip-del-nodo> \
      --node-ip=<ip-del-nodo>
```

### Instalación de un Agent K3s

Para instalar k3s como _agent (agente)_, ejecutamos el siguiente comando:

```bash
curl -sfL https://get.k3s.io | \
  sh \
    -s - agent \
      --server=https://<ip-del-servidor>:6443 \
      --token=<token-del-servidor> \
      --node-ip=<ip-del-agente>
```

Donde el `token-del-servidor` se encuentra en el archivo
`/var/lib/rancher/k3s/server/node-token` en el nodo servidor. Para visualizarla
podemos ejecutar en el nodo servidor el siguiente comando:

```bash
sudo cat /var/lib/rancher/k3s/server/node-token
```
