cargo build
cd target/debug
mpirun -np 4 ./actor_model
cd ../..