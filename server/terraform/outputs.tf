output "application" {
  description = "Object representing the deployed application"
  value       = juju_application.hardware_api
}

output "requires" {
  description = "Map of requires integration endpoints"
  value = {
    ingress = "ingress"
  }
}
