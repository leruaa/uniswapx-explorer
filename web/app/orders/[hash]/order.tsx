"use client"

import { useSuspenseQuery } from '@tanstack/react-query'
import Field from '../../../components/field';
import { Order } from '../../../types/order';
import { OrderDetails } from '../../../types/order-details';

async function getOrder(hash: string): Promise<Order> {
  console.log("order", hash);
  return fetch("http://localhost:3000/data/orders/" + hash + "/data.json")
    .then((response) => response.json())
}

async function getOrderDetails(hash: string): Promise<OrderDetails> {
  let response = await fetch("/data/orders/" + hash + "/details.json");

  if (!response.ok) {
    throw new Error('Network response was not ok')
  }

  return response.json()
}

export default function Order({ hash }: { hash: string }) {
  const { data: order } = useSuspenseQuery({ queryKey: ['order', hash], queryFn: () => getOrder(hash) })
  const { data: details } = useSuspenseQuery({ queryKey: ['orderDetails', hash], queryFn: () => getOrderDetails(hash) })

  const from = order.input.settledAmount || order.input.startAmount;
  const fromUsdValue = from * order.input.price;
  const to = order.output.settledAmount;
  const toUsdValue = to * order.output.price;

  return <>
    <h1 className="text-2xl tabular-nums">{order.hash}</h1>
    <p>From {from} {order.input.symbol} (${fromUsdValue.toFixed(2)}) to {to} {order.output.symbol} (${toUsdValue.toFixed(2)})</p>
    <p>
      <Field title="Created at">{order.createdAt}</Field>
      <Field title="Decay start">{details.decayStartTime}</Field>
      <Field title="Decay end">{details.decayEndTime}</Field>
      <Field title="Deadline">{details.deadline}</Field>
      <Field title="Type">{order.type}</Field>
      <Field title="Status">{order.status}</Field>
      <Field title="Swapper">{details.swapper}</Field>
      <Field title="Recipient">{order.recipient}</Field>
      <Field title="Signature">{order.signature}</Field>
      <Field title="Exclusive filler">{details.exclusiveFiller}</Field>
      <Field title="Reactor">{details.reactor}</Field>
    </p>
  </>
}