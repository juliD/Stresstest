cargo build
cd target/debug
mpirun -np 5 ./actor_model
cd ../..