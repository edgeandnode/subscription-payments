---- MODULE subscriptions ----
EXTENDS Apalache, Integers, Sequences, TLC, prelude

Contract == 0
Owner == 1
Users == 2..5
Addrs == {Contract, Owner} \union Users

MaxBlock == 11
Blocks == 0..MaxBlock

MaxPrice == 3
Prices == 1..MaxPrice

VARIABLES
    \* @type: Int;
    block,
    \* @type: Int -> Int;
    balances,
    \* @typeAlias: subscription = { start: Int, end: Int, price: Int };
    \* @type: Int -> $subscription;
    subs,
    \* @type: Int;
    uncollected

\* @type: (Int, Int, Int) => $subscription;
Sub(start, end, price) == [start |-> start, end |-> end, price |-> price]

\* @type: () => $subscription;
NullSub == Sub(0, 0, 0)

\* @type: ($subscription) => $subscription;
EndSub(sub) == Sub(Min2(block, sub.start), block, sub.price)

\* @type: ($subscription) => $subscription;
TruncateSub(sub) == Sub(Clamp(block, sub.start, sub.end), sub.end, sub.price)

\* @type: (Int) => Bool;
Subscribed(user) == (subs[user].start <= block) /\ (block < subs[user].end)

\* @type: ($subscription) => Int;
SubTotal(sub) == sub.price * (sub.end - sub.start)

\* @type: ($subscription) => Int;
Locked(sub) == sub.price * Max2(0, Min2(block, sub.end) - sub.start)

\* @type: ($subscription) => Int;
Unlocked(sub) == sub.price * Max2(0, sub.end - Max2(block, sub.start))

\* @type: Int;
Collectable == uncollected + ApaFoldSet(LAMBDA sum, user: sum + Locked(subs[user]), 0, Users)

Transfer(from, to, amount) ==
    /\ balances[from] > amount
    /\ balances' := [balances EXCEPT ![from] = @ - amount, ![to] = @ + amount]

Init ==
    /\ block := 0
    /\ balances := [addr \in Addrs |-> Gen(1)] /\ \A b \in Range(balances): b \in Nat
    /\ subs := [user \in Users |-> NullSub]
    /\ uncollected := 0

NextBlock ==
    /\ UNCHANGED <<subs, balances, uncollected>>
    /\ block < MaxBlock
    /\ block' = block + 1

Collect ==
    /\ UNCHANGED <<block>>
    /\ Transfer(Contract, Owner, Collectable)
    /\ uncollected' := 0
    /\ subs' := [user \in Users |-> TruncateSub(subs[user])]

Subscribe(user) ==
    /\ UNCHANGED <<block, uncollected>>
    /\ \E start \in Blocks: \E end \in Blocks: \E price \in Prices: LET sub == Sub(start, end, price) IN
        /\ (block <= start) /\ (start < end)
        /\ subs[user].end <= start
        /\ Transfer(user, Contract, SubTotal(sub))
        /\ subs' := [subs EXCEPT ![user] = sub]

Unsubscribe(user) ==
    /\ UNCHANGED <<block>>
    /\ Subscribed(user)
    /\ Transfer(Contract, user, Unlocked(subs[user]))
    /\ uncollected' := uncollected + Locked(subs[user])
    /\ subs' := [subs EXCEPT ![user] = NullSub]

\* TODO: Resub

Next ==
    \/ NextBlock
    \/ Collect
    \/ \E user \in Users:
        \/ Subscribe(user)
        \/ Unsubscribe(user)

\* @type: (Int -> Int) => Int;
Total(bs) == ApaFoldSet(LAMBDA sum, addr: sum + bs[addr], 0, Addrs)

TypeOK ==
    /\ block \in Nat
    /\ \A b \in Range(balances): b \in Nat
    /\ \A sub \in Range(subs):
        (sub.start \in Nat) /\ (sub.end \in Nat) /\ (sub.start <= sub.end) /\ (sub.price \in Nat)
    /\ uncollected \in Nat

CollectEffect == Collect =>
    /\ balances'[Owner] = balances[Owner] + Collectable
    /\ uncollected' = 0 /\ \A sub \in Range(subs'): Locked(sub) = 0

SubEffect == \A user \in Users: Subscribe(user) =>
    /\ balances'[Contract] = balances[Contract] + SubTotal(subs'[user])
    /\ subs'[user] /= NullSub
    /\ subs'[user].start >= block
    /\ subs'[user].end > block

UnsubEffect == \A user \in Users: Unsubscribe(user) =>
    /\ balances'[user] = balances[user] + Unlocked(subs[user])
    /\ subs'[user].end <= block

Safety ==
    /\ TypeOK
    /\ \A user \in Users: Subscribed(user) => subs[user] /= NullSub
    /\ \A sub \in Range(subs): SubTotal(sub) = (Locked(sub) + Unlocked(sub))
    /\ Total(balances) = Total(balances')
    /\ CollectEffect
    /\ SubEffect
    /\ UnsubEffect

====
