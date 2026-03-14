# Instalación y Despliegue de Kubernetes

## Instalación como Cliente

Una vez lista la VPN, procedemos a instalar y desplegar **k3s**, es necesario
ejecutar el siguiente comando:

```bash
curl -sfL https://get.k3s.io \
    | K3S_URL=https://<control-plane-ip:port> \
        K3S_TOKEN='<k3s-token>' \
        sh -s - agent --node-ip=<node-ip>
```

> [!NOTE]
>
> En nuestro caso la VPN no tiene salida hacia Internet, así que lo ejecutamos
> sin conectarnos a la VPN y una vez finalizada, nos conectamos a la VPN y
> reiniciamos el servicio `k3s-agent`:
>
> ```bash
> sudo systemctl restart k3s-agent
> ```

## Instalación como Servidor

## Comandos

```bash
curl -sfL https://get.k3s.io | sh -s - server \
    --node-ip=10.5.5.2 \
    --flannel-iface=wg0 \
    --flannel-backend=host-gw \
    --cluster-init

curl -sfL https://get.k3s.io | \
      K3S_URL=https://10.5.5.2:6443 \
        K3S_TOKEN='K1025e39a1543ee5ea89ecd9ac6fe3a3b9195605b7b028fe57e3418f710aaca4101::server:0844c472d6479793357e4b9ca066723d' \
        sh -s - agent \
        --node-ip=10.5.5.5 \
        --flannel-iface=wg0 \


# Get token
sudo cat /var/lib/rancher/k3s/server/node-token

# It must be active (running)
sudo systemctl status k3s
sudo systemctl restart k3s

# Show active nodes
sudo k3s kubectl get nodes
sudo k3s kubectl get nodes -o wide

# Apply all manifest from k8s
sudo k3s kubectl apply -t k8s/
# Apply a specific manifest from k8s/
sudo k3s kubectl apply -t k8s/<manifest.yml>

# Show active pods
sudo k3s kubectl get pods
sudo k3s kubectl get pods -o wide

# Describe a pod
sudo k3s kubectl describe pod worker-0

# Describe all pods
sudo k3s kubectl delete pods --all
sudo k3s kubectl delete pods --all --force --grace-period=0

# Delete some problematic manifest
sudo k3s kubectl delete statefulset worker
sudo k3s kubectl delete deployment coordinator

# Show images
sudo k3s crictl images
sudo k3s crictl images | grep quantum

# Coordinator logs
kubectl logs -f deployment/coordinator
# Container-0 logs
kubectl logs container-0
```
