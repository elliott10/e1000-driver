#!/bin/bash

ip link set eno1 nomaster
ifdown eno1
ip link set enp3s0 master bridge0
ifup enp3s0

brctl show
