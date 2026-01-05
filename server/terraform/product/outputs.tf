output "hardware_api_app_name" {
  description = "The name of the Hardware API application"
  value       = module.hardware_api.app_name
}

output "hardware_api_requires" {
  value = {
    nginx_route = "nginx-route"
  }
}

output "hardware_api_provides" {
  value = {}
}

output "nginx_ingress_integrator_app_name" {
  description = "The name of the NGINX Ingress Integrator application"
  value       = juju_application.nginx_ingress_integrator.name
}

output "nginx_ingress_integrator_requires" {
  value = {}
}

output "nginx_ingress_integrator_provides" {
  value = {
    nginx_route = "nginx-route"
  }
}
