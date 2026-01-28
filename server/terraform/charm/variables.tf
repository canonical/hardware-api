variable "app_name" {
  type        = string
  default     = "hwapi"
  description = "Name to give the deployed application"
}

variable "base" {
  type        = string
  nullable    = true
  default     = null
  description = "The operating system on which to deploy"
}

variable "channel" {
  type        = string
  default     = "latest/edge"
  description = "Channel of the charm"
}

variable "config" {
  type        = map(string)
  default     = {}
  description = "Map for config options"
}

variable "constraints" {
  type        = string
  default     = "arch=amd64"
  description = "String listing constraints for this application"
}

variable "model" {
  type        = string
  nullable    = true
  default     = null
  description = "Reference to an existing model resource or data source for the model to deploy to"
}

variable "revision" {
  type        = number
  nullable    = true
  default     = null
  description = "Revision number of the charm"
}

variable "units" {
  type        = number
  default     = 1
  description = "Unit count/scale"
}
