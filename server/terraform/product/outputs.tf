output "applications" {
  value = {
    hardware_api = module.hardware_api
    traefik_k8s  = juju_application.traefik_k8s
  }
}
