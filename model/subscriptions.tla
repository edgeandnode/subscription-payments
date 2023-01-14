---- MODULE subscriptions ----
EXTENDS Apalache, Integers, Sequences, TLC, prelude

Contract == 0
Owner == 1
Users == 2..5 \* TODO: check if Contract & Owner can be users
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

\* @type: ($subscription, Int) => $subscription;
ExtendSub(sub, end) == Sub(sub.start, Max2(end, sub.end), sub.price)

\* @type: ($subscription) => Int;
SubTotal(sub) == sub.price * (sub.end - sub.start)

\* @type: ($subscription, Int) => Int;
LockedAt(sub, block_) == sub.price * Max2(0, Min2(block_, sub.end) - sub.start)

\* @type: ($subscription) => Int;
Locked(sub) == LockedAt(sub, block)

\* @type: ($subscription, Int) => Int;
UnlockedAt(sub, block_) == sub.price * Max2(0, sub.end - Max2(block_, sub.start))

\* @type: ($subscription) => Int;
Unlocked(sub) == UnlockedAt(sub, block)

\* @type: Int;
Collectable == uncollected + ApaFoldSet(LAMBDA sum, user: sum + Locked(subs[user]), 0, Users)

Transfer(from, to, amount) ==
    /\ balances[from] > amount
    /\ balances' := [balances EXCEPT ![from] = @ - amount, ![to] = @ + amount]

Init ==
    /\ block := Gen(1) /\ block \in Blocks
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
        /\ subs[user].end <= block
        /\ Transfer(user, Contract, SubTotal(sub))
        /\ subs' := [subs EXCEPT ![user] = sub]

Unsubscribe(user) ==
    /\ UNCHANGED <<block>>
    /\ Transfer(Contract, user, Unlocked(subs[user]))
    /\ uncollected' := uncollected + Locked(subs[user])
    /\ subs' := [subs EXCEPT ![user] = NullSub]

Extend(user) ==
    /\ UNCHANGED <<block, uncollected>>
    /\ \E end \in Blocks: LET sub == ExtendSub(subs[user], end) IN
        /\ (subs[user].start <= block) /\ (block < subs[user].end)
        /\ Transfer(user, Contract, SubTotal(sub) - SubTotal(subs[user]))
        /\ subs' := [subs EXCEPT ![user] = sub]

Next ==
    \/ NextBlock
    \/ Collect
    \/ \E user \in Users:
        \/ Subscribe(user)
        \/ Unsubscribe(user)
        \/ Extend(user)

\* @type: (Int -> Int) => Int;
Total(bs) == ApaFoldSet(LAMBDA sum, addr: sum + bs[addr], 0, Addrs)

TypeOK ==
    /\ block \in Blocks
    /\ \A b \in Range(balances): b \in Nat
    /\ \A sub \in Range(subs): (sub = NullSub) \/
        /\ (sub.start \in Blocks) /\ (sub.end \in Blocks) /\ sub.start <= sub.end
        /\ sub.price \in Prices
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

ExtendEffect == \A user \in Users: Extend(user) =>
    /\ UnlockedAt(subs'[user], block') >= Unlocked(subs[user])
    /\ LockedAt(subs'[user], block') = Locked(subs[user])

Safety ==
    /\ TypeOK
    /\ \A sub \in Range(subs): SubTotal(sub) = (Locked(sub) + Unlocked(sub))
    /\ Total(balances) = Total(balances')
    /\ CollectEffect
    /\ SubEffect
    /\ UnsubEffect
    /\ ExtendEffect
    \* The balance and recoverable (unlocked) value for a user can't drop by more than the
    \* subscription price per step.
    /\ \A user \in Users:
        (balances'[user] + UnlockedAt(subs'[user], block')) >=
        (balances[user] + (Unlocked(subs[user]) - subs[user].price))

====
