
#cd rust && cargo build && cd .. # Rust Projekt bauen

# Rust Build erneuern
rm -rf ./docker/docker-image/rust-build
cp -rf rust/ docker/docker-image/rust-build

# Docker Netzwerk aufbauen
cd ./docker/
docker-compose down -v
docker rm $(docker ps -a -q)
docker-compose build
docker-compose up --scale node=4 -d

#
ssh -p 2222 -i docker-image/ssh-keys/id_rsa.mpi -o StrictHostKeyChecking=no mpi_user@localhost './make-hostfile.sh'
ssh -p 2222 -i docker-image/ssh-keys/id_rsa.mpi -o StrictHostKeyChecking=no mpi_user@localhost 'mpirun -np 16 rust-build/target/debug/actor_model'

# Auf den Master verbinden
#sh connect-master.sh