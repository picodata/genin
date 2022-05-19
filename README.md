# GENIN

- [GENIN](#genin)
    * [Installation](#installation)
        - [RHEL, Fedora, Rockylinux](#rhel--fedora--rockylinux)
        - [Debian, Ubuntu](#debian--ubuntu)
        - [Apple MacOs](#apple-macos)
        - [Windows](#windows)
    * [Usage guide](#usage-guide)
        + [Inventory generation](#inventory-generation)
        + [Flags and options](#flags-and-options)
    * [Build from sources](#build-from-sources)
    * [Contributing](#contributing)
    * [Versioning](#versioning)
    * [Authors](#authors)
    * [License](#license)

---
## Installation

Download and unzip the archive for the desired architecture.

#### RHEL, Fedora, Rockylinux
Target distribution RHEL, Fedora, Rockylinux has two installation methods.
1. Installation using the package manager.
Call this command with sudo rights:
```shell
sudo bash -c 'cat <<\EOF >> /etc/yum.repos.d/picotools.repo
[picodata.io]
name=picodata.io public repository
baseurl=https://binary.picodata.io/repository/yum/el/$releasever/$basearch/os/
enabled=1
gpgcheck=0
EOF'
```
And now just install **genin**:
```shell
yum install -y genin
```

2. If you want to download and install manually please don't forget chose right architecture:
```shell
rpm -i https://binary.picodata.io/repository/yum/el/8/x86_64/os/genin-0.3.1-1.el8.x86_64.rpm
```

#### Debian, Ubuntu
It is also possible to install a `deb` package on `debian` based distros:
```shell
curl -sLO https://binary.picodata.io/repository/raw/genin/deb/genin-0.3.1.amd64.deb && sudo dpkg -i genin-0.3.1.amd64.deb
```

#### Apple MacOs
```shell
curl -L https://binary.picodata.io/repository/raw/genin/apple/genin-0.3.1-darwin-amd64.zip -o genin-0.3.1-darwin-amd64.zip 
unzip genin-0.3.1-darwin-amd64.zip -d ~/bin/
```

#### Windows
```shell
curl.exe -L https://binary.picodata.io/repository/raw/genin/windows/genin-0.3.1-darwin-amd64.zip -o genin-0.3.1-windows-amd64.zip 
unzip.exe genin-0.3.1-windows-amd64.zip -d %HOME%/.cargo/bin/
```

Check version:
```
genin --version
```

---
## Usage guide

### Inventory generation

First, let's generate a simple cluster for the `Vagrant`.  
`Genin` works on the basis of a configuration in the form of a `yaml` file in which the
cluster parameters will be minimally described. This file has its own description principle.
So for the simplicity we use `genin` built-in generation of this file. Just enter subcommand
`init`.
```shell
genin init
```
As a result, a file `cluster.genin.yaml` will be created in the same directory where `genin`
was launched.
> **Note:** if the directory in which the initialization was started already contains a
> `cluster.genin.yml`, then will be created new file `cluster.genin1.yaml` and
> `genin` will create a new one.

Let's check it! Open the file and examine the contents.
```yaml
---
# list of instances as an array
instances:
  # instance looks like item in array
  - name: storage # (mandatory) instance name
    type: storage       # (mandatory) instance type (storage, router, custom)
    count: 2            # (optional) how many masters we want, by default equal 1
    replicas: 0         # (optional) number of replicas per master, default 0
    weight: 10          # (optional) instance weight
    roles: ["storage", "failover-coordinator"] # (optional) list of roles    
    config:             # (optional) config with arbitrary key-values pairs
      my_key: my_value  # please do not leave empty blocks

  # all instances generated using the init subcommand will have the same set of parameters
  - name: router
    type: storage
    count: 1

  # this one will be type "custom" and count 1
  - name: engine
  # here you can add your own instance

# vars similar to those configured in the TDG inventory
vars:
  ansible_user: my_user
  ansible_password: my_user_password
  cartridge_app_name: my_app
  cartridge_package_path: /tmp/my_app.rpm
  cartridge_cluster_cookie: my_app_cluster_cookie

failover:
  mode: stateful
  state_provider: stateboard
  stateboard_params:
      uri: "192.168.16.11:3033"
      password: "vG?-GG!4sxV8q5:f"

# map of datacenters and hosts
hosts: # (important) all dc's is a key: map objects
  - name: kavkaz        # (important) at least one datacenter must be designated
    type: region        # (optional) host type
    distance: 10
    ports:              # (optional) begin binary and http port, by default 8080, 3030
      http: 8091        # ports can be defined on all levels (region, datacenter, server)
      binary: 3031
    hosts:
      - name: dc-1
        type: datacenter
        hosts:  # (important) without defined type it will be servers
          - name: server-1
            ip: 10.20.3.100
      - name: dc-2
        type: datacenter
        hosts:
          - name: server-1
            ip: 10.20.4.100
  - name: moscow
    type: region
    distance: 20
    hosts:
      - name: dc-3
        type: datacenter
        ports:
          http: 8091
          binary: 3031
        hosts:
          - name: server-10
            ip: 10.99.3.100
```
After you have changed those parameters that you needed, save it.
Half of the work is done, it remains only to generate a ready-made inventory from this
file. But before doing this, there are a few more configuration options to mention.

Now let's run initialization with additional flags.
```shell
genin init --output mycluster.yml
```
> **Note:** Note that the path where the finished cluster file will be saved can be set with
> the `--output` flag.

Now let's check what is the difference from the previous initialization `mycluster.yml`.
```yaml
---
---
instances:
  - name: router
    type: router
    count: 1
    replicas: 0
    weight: 10
    roles:
      - router
      - api
      - failover-coordinator
  - name: storage
    type: storage
    count: 2
    replicas: 2
    weight: 10
    roles:
      - storage
hosts:
  - name: selectel
    type: datacenter
    ports:
      http: 8081
      binary: 3031
    hosts:
      - name: host-1
        ip: 192.168.16.1
      - name: host-2
        ip: 192.168.16.2
failover:                       # (important) failover parameters
  mode: stateful                # (important) only stateful mode has state provider. By default, stateful.
  state_provider: stateboard    # (important) state provider can be etcd2 or stateboard
  stateboard_params:
    uri:
      ip: 192.168.16.1          # (optional) listening address
      port: 4401                # (optional) listening port
    password: change_me         # (optional) stateboard password should be same as in stateboard params
vars:
  ansible_user: root
  ansible_password: change_me
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
```

The difference is not so great, but it is there since we launched it with additional parameters.  
These parameters added to us `cartridge_failover_params` and `stateboard` instance.
Now run `genin` in the same folder where `cluster.genin.yml` is stored. You can read more about  
such failover modes in the [Tarantool](https://www.tarantool.io/ru/doc/1.10/book/cartridge/topics/failover/) documentation.

But the main thing is that now nothing prevents us from generating inventory. Let's do it!
```shell
genin build --print
```
> **Note:** Print flag only neaded to better visibility of result

Command `build` signalize that `genin` should build inventory.
Flag `--print` (short `-s`) will additionally show us the distribution of instances by hosts.
If the program did not return any errors, then our inventory was successfully generated and
written to disk in the same  directory where `genin` was launched.
Time to check result!
```yaml
TODO
```

### Flags and options

First, you can always change the path to the file with the cluster configuration.
The `--source` (short `-s`) and `--output` (short `-o`) flags serve this purpose.
It is recommended to use the `.genin.` suffix for naming convenience.
```shell
genin build --source /home/user/path/my_cluster.genin.yml --output /home/tarantool/cluster-new/my_hosts.yml
genin build -s /home/user/path/my_cluster.genin.yml -o /home/tarantool/cluster-new/my_hosts.yml
```

The next, of course, a very useful option is verbose output of logs. You can
control it with the flags `-v` from one to three `-vvv`. Single `v` means *INFO*, double means
*DEBUG*, and three or more means *TRACE*.
```
genin build -vvv
```
> **Note:** All commands without source and output options always check for files in the current 
> directory (cluster.genin.yaml, inventory.yaml)

The `--print-opts` (short `-p`) flag will allow you to select the print output options. By default, 
only the distribution of instances by host will be shown.
```shell
genin build -s colorized, ports
```

Sometimes it can be useful to quickly change the `failover-mode` using flag without changing
the cluster configuration, or on initial step. Possible variants `statefull`, `eventual`, 
`disabled` (by default `statefull`).
```shell
genin init --failover-mode eventual
```

> **Note:** This option works with *failover-state-provider*. When choosing a failover mode
> *stateful*, you can set the *failover-state-provider* (stateboard or etcd2).

```shell
genin init --failover-mode stateful --failover-state-provider etcd2
```

You can also set sensitive information using the options. For example user and password for 
server there you're deploying cluster, or cluster cookie.
```shell
genin build --ansible-user dmitry.travyan --ansible-password ddfqd
genin build --cartridge-cluster-cookie R68sJfV4C2hLrWC3
```

## Build from sources

At first, you need to clone the source code.
```shell
git clone git@github.com:picodata/genin.git
```

Second you should install rust build tools.

Debian like, Centos, RHEL, Fedora, Macos.
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
> **Note:** You should refresh `$PATH` variable for access to calling `rust binaries`.

After install all required tools, we should build and install `genin`.
```
cd genin
rustup override set nightly
cargo +nightly build --release
cargo +nightly install --path .
```

> **Note:** Do not forget to install build tools before building `genin`.
>
> Debian like:
> ```shell
> sudo apt install -y build-essential
> ```
> RHEL, Centos, Fedora
> ```
> sudo yum install -y gcc
> ```
> No additional action is required for macOS.

And now let's check the version.
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

This project is licensed under the BSD License - see the [LICENSE](LICENSE) file for details
