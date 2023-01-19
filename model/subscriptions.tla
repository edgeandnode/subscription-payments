---- MODULE subscriptions ----
EXTENDS Apalache, Integers, Sequences, TLC, prelude

Contract == 0
Owner == 1
Addrs == {Contract, Owner} \union (2..4)

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
TruncateSub(sub) == LET start == Clamp(block, sub.start, sub.end) IN
    IF start = sub.end THEN NullSub ELSE Sub(start, sub.end, sub.price)

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
Collectable == uncollected + ApaFoldSet(LAMBDA sum, addr: sum + Locked(subs[addr]), 0, Addrs)

Transfer(from, to, amount) ==
    /\ balances[from] > amount
    /\ balances' := [balances EXCEPT ![from] = @ - amount, ![to] = @ + amount]

Init ==
    /\ block := Gen(1) /\ block \in Blocks
    /\ balances := [addr \in Addrs |-> Gen(1)] /\ \A b \in Range(balances): b \in Nat
    /\ subs := [addr \in Addrs |-> NullSub]
    /\ uncollected := 0

NextBlock ==
    /\ UNCHANGED <<subs, balances, uncollected>>
    /\ block < MaxBlock
    /\ block' = block + 1

Collect ==
    /\ UNCHANGED <<block>>
    /\ Transfer(Contract, Owner, Collectable)
    /\ uncollected' := 0
    /\ subs' := [addr \in Addrs |-> TruncateSub(subs[addr])]

Subscribe(addr) ==
    /\ UNCHANGED <<block, uncollected>>
    /\ addr /= Contract
    /\ \E start \in Blocks: \E end \in Blocks: \E price \in Prices: LET sub == Sub(start, end, price) IN
        /\ (block <= start) /\ (start < end)
        /\ subs[addr].end <= block
        /\ Transfer(addr, Contract, SubTotal(sub))
        /\ subs' := [subs EXCEPT ![addr] = sub]

Unsubscribe(addr) ==
    /\ UNCHANGED <<block>>
    /\ Transfer(Contract, addr, Unlocked(subs[addr]))
    /\ uncollected' := uncollected + Locked(subs[addr])
    /\ subs' := [subs EXCEPT ![addr] = NullSub]

Extend(addr) ==
    /\ UNCHANGED <<block, uncollected>>
    /\ \E end \in Blocks: LET sub == ExtendSub(subs[addr], end) IN
        /\ (subs[addr].start <= block) /\ (block < subs[addr].end)
        /\ Transfer(addr, Contract, SubTotal(sub) - SubTotal(subs[addr]))
        /\ subs' := [subs EXCEPT ![addr] = sub]

Next ==
    \/ NextBlock
    \/ Collect
    \/ \E addr \in Addrs:
        \/ Subscribe(addr)
        \/ Unsubscribe(addr)
        \/ Extend(addr)

\* @type: (Int -> Int) => Int;
Total(bs) == ApaFoldSet(LAMBDA sum, addr: sum + bs[addr], 0, Addrs)

Users == Addrs \ {Contract}

TypeOK ==
    /\ block \in Blocks
    /\ \A b \in Range(balances): b \in Nat
    /\ \A sub \in Range(subs): (sub = NullSub) \/
        /\ (sub.start \in Blocks) /\ (sub.end \in Blocks) /\ sub.start < sub.end
        /\ sub.price \in Prices
    /\ uncollected \in Nat

CollectEffect == Collect =>
    /\ balances'[Owner] = balances[Owner] + Collectable
    /\ uncollected' = 0 /\ \A sub \in Range(subs'): Locked(sub) = 0

SubEffect == \A addr \in Addrs: Subscribe(addr) =>
    /\ balances'[Contract] = balances[Contract] + SubTotal(subs'[addr])
    /\ subs'[addr] /= NullSub
    /\ subs'[addr].start >= block
    /\ subs'[addr].end > block

UnsubEffect == \A addr \in Addrs: Unsubscribe(addr) =>
    /\ balances'[addr] = balances[addr] + Unlocked(subs[addr])
    /\ subs'[addr].end <= block

ExtendEffect == \A addr \in Addrs: Extend(addr) =>
    /\ UnlockedAt(subs'[addr], block') >= Unlocked(subs[addr])
    /\ LockedAt(subs'[addr], block') = Locked(subs[addr])

Safety ==
    /\ TypeOK
    /\ subs[Contract] = NullSub
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
