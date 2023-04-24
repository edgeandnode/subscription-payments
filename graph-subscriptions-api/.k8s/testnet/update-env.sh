./auth.sh
kubectl apply -f env.yml
kubectl rollout restart deploy/graph-subscriptions-api