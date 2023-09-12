Name: genin
Version: 0.5.5
Release: 1%{?dist}
URL: https://github.com/picodata/genin
Epoch: 1

Summary: Simple template engine that makes it easy to create inventory for tarantool cartridge
License: BSD-2
Group: Databases

Source0: %name-%version.tar.gz

%description
Simple template engine that makes it easy to create inventory for tarantool cartridge

%prep
%setup -q -n %{name}-%{version}

%build
make install-cargo
make build

%install
%if "%{?_build_vendor}" == "alt"
%makeinstall_std
%else
%make_install
%endif

%files
%{_bindir}/genin
%doc README.md README.ru.md
%{!?_licensedir:%global license %doc}
%if "%{?_build_vendor}" == "alt"
%doc LICENSE AUTHORS
%else
%license LICENSE AUTHORS
%endif

%changelog
* Mon Sep  4 2023 <kdy@picodata.io> - 0.5.5%{?dist}
   - Add build rpm for many OS
