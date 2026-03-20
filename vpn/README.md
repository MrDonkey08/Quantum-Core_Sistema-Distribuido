# Configuración WireGuard

## IPs asignadas por nodo
| Integrante | Hostname        | IP WireGuard |
|------------|-----------------|--------------|
| Elisa      | elisapc         | 10.5.5.5     |
| Peer 1     | ...             | 10.5.5.1     |
| Peer 2     | desktop-7v7jtd4 | 10.5.5.2     |
| Peer 3     | debian          | 10.5.5.4     |
| Peer 4     | juampa          | 10.5.5.8     |
| Peer 5     | msi             | 10.5.5.6     |

## Generar claves
```bash
# Generar clave privada
wg genkey | tee privatekey

# Generar clave pública desde la privada
cat privatekey | wg pubkey > publickey
```

## Pasos para unirse al cluster
1. Instalar WireGuard
2. Copiar `client-wg0.conf.example` a `/etc/wireguard/wg0.conf`
3. Reemplazar los valores entre `< >`
4. Ejecutar `sudo wg-quick up wg0`
5. Verificar con `sudo wg show`

## Verificar conectividad
```bash
ping 10.5.5.5
sudo wg show
```
