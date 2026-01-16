output "hardware_api_app_name" {
  description = "The name of the Hardware API application"
  value       = module.hardware_api.app_name
}

output "hardware_api_requires" {
  value = {}
}

output "hardware_api_provides" {
  value = {}
}

output "traefik_app_name" {
  description = "The name of the Traefik application"
  value       = juju_application.traefik.name
}

output "traefik_requires" {
  value = {
    certificates = "certificates"
  }
}

output "traefik_provides" {
  value = {}
}

output "lego_app_name" {
  description = "The name of the Lego application"
  value       = juju_application.lego.name
}

output "lego_requires" {
  value = {}
}

output "lego_provides" {
  value = {
    certificates = "certificates"
  }
}
