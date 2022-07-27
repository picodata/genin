 GENIN
---
![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/picodata/genin)
![GitHub Workflow Status (branch)](https://img.shields.io/github/workflow/status/picodata/genin/IntegrationTest/master?label=test&logo=test)
[![License](https://img.shields.io/badge/License-BSD_2--Clause-orange.svg)](LICENSE)
[![en](https://img.shields.io/badge/lang-en-cyan.svg)](https://github.com/picodata/genin/blob/master/README.md)
[![ru](https://img.shields.io/badge/lang-ru-green.svg)](https://github.com/picodata/genin/blob/master/README.ru.md)

- [GENIN](#genin)
    * [About](#about)
    * [Installation](#installation)
        - [RHEL, Fedora, Rockylinux](#rhel-centos-rockylinux-fedora)
        - [Debian, Ubuntu](#debian-ubuntu)
        - [macOS](#macos)
        - [Windows](#windows)
    * [Usage guide](#usage-guide)
        + [Inventory generation](#inventory-generation)
        + [Flags and options](#flags-and-options)
    * [Building from sources](#building-from-sources)
    * [Contributing](#contributing)
    * [Versioning](#versioning)
    * [Authors](#authors)
    * [License](#license)

---

## About
Genin is an inventory generator for Ansible Cartridge. It provides a command-line 
tool that allows quick inventory creation for clusters of any size.
For example, an inventory file for a cluster of 50 instances can easily be of thousand 
lines or more. Any slight change of the configuration, eg. adding a new configuration 
option for all storages, means a lot of manual routine and increases the risk of 
improper or incomplete configuration. Genin  allows you to stay confident while 
maintaining the configuration file and steer clear of inaccuracies and human errors. 
Genin is the tool that will help you very quickly roll out cluster configuration updates.

## Installation

Download and unzip the archive for the desired architecture.

#### RHEL, CentOS, Rockylinux, Fedora
There are two installation methods supported for RHEL, CentOS, Rockylinux and Fedora.

1. Installation using the package manager.

Copy and paste the following command into your terminal window:
```shell
sudo bash -c 'cat <<\EOF >> /etc/yum.repos.d/picotools.repo
[picodata.io]
name=picodata.io public repository
baseurl=https://binary.picodata.io/repository/yum/el/$releasever/$basearch/os/
enabled=1
gpgcheck=0
EOF'
```
After that install **Genin**:
```shell
sudo yum install -y genin
```

2. If you want to install `rpm` packages directly without adding our repository.
```shell
# RHEL 8.x, CentOS 8.x, Rockylinux 8.x, recent Fedora version
sudo rpm -i https://binary.picodata.io/repository/yum/el/8/x86_64/os/genin-0.3.6-1.el8.x86_64.rpm
# RHEL 7.x, CentOS 7.x
sudo rpm -i https://binary.picodata.io/repository/yum/el/7/x86_64/os/genin-0.3.6-1.el7.x86_64.rpm
```
> **Note:** please don't forget to pick the right package for your OS version.

#### Debian, Ubuntu
We provide the `deb` Genin package for `debian`-based Linux distributions including the Ubuntu family. Use the following command to download and install the package:
```shell
curl -sLO https://binary.picodata.io/repository/raw/genin/deb/genin-0.3.6.amd64.deb && sudo dpkg -i genin-0.3.6.amd64.deb
```

#### macOS
Use the following command to grab and install Genin in macOS (10.10+):
```shell
curl -L https://binary.picodata.io/repository/raw/genin/apple/genin-0.3.6-darwin-amd64.zip -o genin-0.3.6-darwin-amd64.zip 
unzip genin-0.3.6-darwin-amd64.zip -d ~/bin/
```
> **Note:** The application can then be found under the `~/bin` directory. 
> Make sure the directory is in your `$PATH`.

#### Windows
Use the following command to grab and install Genin in Windows 7 64 bit or newer:
```shell
curl.exe -L https://binary.picodata.io/repository/raw/genin/windows/genin-0.3.6-darwin-amd64.zip -o genin-0.3.6-windows-amd64.zip 
unzip.exe genin-0.3.6-windows-amd64.zip -d %HOME%/.cargo/bin/
```
> **Note:** The application can then be found under the `.cargo/bin` folder inside 
> your user profile folder. Make sure it is in your `%PATH%`.

Сheck that the installation was successful:
```
genin --version
```

---
## Usage guide

### Inventory generation
 
First, let's generate a simple cluster for the `Vagrant` virtual environment. 
For that `Genin` will need a `yaml` file with a concise list of cluster details. That is a minimal cluster 
configuration file that features `Genin's` own formatting. As long as users will likely need to have a descriptive template of that file, `Genin` can automatically generate it with a built-in dedicated subcommand:

```shell
genin init
```
This will result in creating the `cluster.genin.yaml` file in the current directory.

> **Note:** If the `cluster.genin.yml` file already exists in current directory, then the new file will be named `cluster.genin.copy.yaml`.
> The `genin init` command will always append the `.copy` suffix to the file's name if the expected file exists.

Also, you can explicitly set the configuration file name:

```shell
genin init --output mycluster.yml
```
> **Note:** Use the `--output` flag together with the full path to `mycluster.yml` to specify the directory where the final cluster files will be saved.

Now you can open the file and examine the syntax.

```yaml
---
# list of instances as an array
instances:
  # instance looks like item in array
  - name: router              # (mandatory) instance name
    type: router              # (mandatory) instance type (storage, router, custom, dummy, replica)
    count: 1                  # (optional) how many masters we want, by default equal 1
    replicas: 0               # (optional) number of replicas per master, default for router 0
    weight: 10                # (optional) instance weight
    roles:                    # (optional) list of roles    
      - router
      - api
      - failover-coordinator
    config:                   # (optional) config with arbitrary key-values pairs
      instance_name: router   # any other configuration parameters in free order

  # all another instances generated using the init subcommand will have the same set of parameters
  - name: storage
    type: storage
    count: 2
    replicas: 2
    weight: 10
    roles:
      - storage

# map of regions, datacenters, and hosts
hosts:                    # (important) at least one datacenter must be designated
  - name: selectel        # (important) at least one datacenter must be designated
    type: datacenter      # (optional) host type
    ports:                # (optional) begin binary and http port, by default 8080, 3030
      http: 8081          # ports can be defined on all levels (region, datacenter, server)
      binary: 3031
    hosts: 
      - name: host-1      # deepest level of hosts config
        ip: 192.168.16.11
      - name: host-2
        ip: 192.168.16.12

# failover parameters
failover:
  mode: stateful                      # (optional) failover mode (stateful, eventual, disabled)
  state_provider: stateboard          # (optional) what is serve failover (stateboard, stateful)
  stateboard_params:                  # (optional) params for chosen in state_provider failover type
      uri:
        ip: 192.168.16.1
        port: 4401
      password: "vG?-GG!4sxV8q5:f"

# vars similar to those configured in the TDG inventory
vars:
  ansible_user: my_user
  ansible_password: my_user_password
  cartridge_app_name: my_app
  cartridge_package_path: /tmp/my_app.rpm
  cartridge_cluster_cookie: my_app_cluster_cookie
  # put here you personally key/value ansible cartridge vars
```

Replace the stubs with the actual values of your hosts and their parameters and save the file.


So far you are already half way through getting things done! Use the resulted `Genin` configuration file to generate the final inventory file.
Here is the required command for that:

```shell
genin build
```

Done! The `inventory.yaml` file will appear in the same directory where you launched `Genin`. Now we 
can set up the cluster:

```shell
ansible-playbook -i inventory.yaml playbook.yaml
```

### Editing config

The initial cluster configuration file can be slimmed down to the following minimal variant:

```yaml
---
instances:
  - name: router
    type: router
  - name: storage
    type: storage
    count: 3
    replicas: 2

hosts:
  - name: selectel
    type: datacenter
    hosts:
      - name: host-1
        ip: 192.168.16.11
      - name: host-2
        ip: 192.168.16.12
```

This is a perfectly valid and working configuration file. The rest of the parameters wil use their default values.
 
Let's now extend the configuration file with a more real-world example featuring 10 hosts, 10 routers, 10 storages, and a default number of storage 
replicas (1). We will also define a different instance type - `cache`.

```yaml
---
instances:
  - name: router
    type: router
    count: 10
  - name: storage
    type: storage
    count: 10

hosts:
  - name: selectel
    type: datacenter
    hosts:
      - name: host-1
        ip: 192.168.16.11
      - name: host-2
        ip: 192.168.16.12
      - name: host-3
        ip: 192.168.16.13
      - name: host-4
        ip: 192.168.16.14
      - name: host-5
        ip: 192.168.16.15
      - name: host-6
        ip: 192.168.16.16
      - name: host-7
        ip: 192.168.16.17
      - name: host-8
        ip: 192.168.16.18
      - name: host-9
        ip: 192.168.16.19
      - name: host-10
        ip: 192.168.16.20
```
The actual difference between the 2 instances configuration and a large cluster configuration
is not that great, whereas the resulting inventory file for the large cluster will be 5 times bigger.

Let's take a look at few more helpful configuration flags, this time regarding the failover capability:

```shell
genin init --failover-mode disabled
```
The above option completely removes all failover parameters for all stages.

```shell
genin build --failover-state-provider etcd2 # eg genin build -F etcd2
```
The above option will redefine the type of the entity serving the failover.

For more options see the [Tarantool documentation](https://www.tarantool.io/ru/doc/1.10/book/cartridge/topics/failover/).

> **Note:** Failover options work for all `genin` subcommands.

### Flags and options

Here we describe few other useful flags and options that you might want to use with `genin`. 
First, you can always change paths of both source and target configuration files usin the `--source` (short `-s`) and `--output` (short `-o`) flags respectively.
We recommend using the `.genin.` suffix for naming convenience. Here is the example command in two variants:
```shell
genin build --source /home/user/path/my_cluster.genin.yml --output /home/tarantool/cluster-new/my_hosts.yml
genin build -s /home/user/path/my_cluster.genin.yml -o /home/tarantool/cluster-new/my_hosts.yml
```

Next, there is a very useful option for controlling the log output. There are three supported log levels that you can enable using the `-v` flag with a desired number of extra `v` letters for more verbosity (up to `-vvv`). Single `v` means *INFO*, double means
*DEBUG*, and three or more mean *TRACE*.
```
genin build -vvv
```
> **Note:** All commands without source and output options always check for files in the current 
> directory (namely cluster.genin.yaml, inventory.yaml)

The `--print-opts` (short `-p`) flag allows you to select the print output options. By default, 
only the distribution of instances over the hosts is shown.


Sometimes it can be useful to quickly change the `failover-mode` using the flag without changing
the cluster configuration. This can be done during the first initialization stage (`genin init`), or later on with other `Genin` subcommands. Possible variants of the flag values are `stateful` (default), `eventual`, 
`disabled`.
```shell
genin init --failover-mode eventual
```

> **Note:** Setting the failover mode to *stateful* allows using the *failover-state-provider* flag (possible values are `stateboard` or `etcd2`).

```shell
genin init --failover-mode stateful --failover-state-provider etcd2
```

You can also provide personal information or credentials using these options, such as user and 
password for the server where the cluster is being deployed, or the cluster cookie.
```shell
genin build --ansible-user dmitry.travyan --ansible-password ddfqd
genin build --cartridge-cluster-cookie R68sJfV4C2hLrWC3
```

As we have known from earlier paragraphs, by default `Genin` will create the copy of the file 
if the target file already exists in the specified path. Use the`--force` flag (or short `-f`) 
to explicitly overwrite the target.
```shell
genin build -o my-cluster.yml
genin build -o my-cluster.yml --force
```

## Building from sources

At first you need to clone the source code.
```shell
git clone https://github.com/picodata/genin.git
```

Second, you need to install Rust build tools.
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
> **Note:** You should refresh the `$PATH` variable to get access to locally installed `rust binaries`.

After installing all required tools it is time to build and install `genin`.
```shell
cd genin
rustup override set nightly
cargo +nightly build --release
install -m 001 target/release/genin /usr/local/bin/
```

> **Note:** Do not forget to install build tools and dependencies before building `genin`.
>
> For Debian-based distributions:
> ```shell
> sudo apt install -y build-essential
> ```
> For Red Hat-based distributions (RHEL, CentOS, Fedora):
> ```
> sudo yum install -y gcc
> ```
> For macOS make sure you have `Command Line Developer Tools` installed (`xcode-select --install`).

Сheck that the installation was successful:
```shell
genin --version
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/picodata/genin/tags).

## Authors

- **Dmitry Travyan**

© 2020-2022 Picodata.io https://github.com/picodata

## License

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details.
