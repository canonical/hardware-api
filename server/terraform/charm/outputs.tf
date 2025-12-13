output "app_name" {
  description = "Name of the deployed application"
  value       = juju_application.hardware_api.name
}

output "endpoints" {
  description = "Map of all endpoints"
  value = {
    nginx_route = "nginx-route"
  }
}
