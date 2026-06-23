output "application" {
  description = "Name of the deployed application"
  value       = juju_application.hardware_api
}

output "requires" {
  description = "Map of requires integration endpoints"
  value = {
    nginx_route = "nginx-route"
    ingress     = "ingress"
  }
}
