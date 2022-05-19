# ---
# Main configuration parameters
# ---
# export TF_VAR_sel_account=
# export TF_VAR_sel_token=
# export TF_VAR_user_name=
# export TF_VAR_user_password=
variable "sel_account" {
  type = string
}

variable "sel_token" {
  type = string
}

variable "user_name" {
  type = string
}

variable "user_password" {
  type = string
}

variable "os_auth_url" {
  default = "https://api.selvpc.ru/identity/v3"
}

variable "project_id" {
  default = "26001df9c2144fdcad88f361cdc2f593"
}

variable "os_region" {
  default = "ru-7"
}

variable "server_zone" {
  default = "ru-7a"
}

# ---
# Target genin_hosts config
# ---
variable "subnet_cidr" {
  default = "192.168.16.0/24"
}

variable "genin_hosts_count" {
  default = 2
}

variable "genin_hosts_vcpus" {
  default = 1
}

variable "genin_hosts_ram_mb" {
  default = 2048
}

variable "genin_hosts_root_disk_gb" {
  default = 10
}
