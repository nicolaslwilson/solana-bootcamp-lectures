const { serialize, deserialize, deserializeUnchecked } = require("borsh");

// Flexible class that takes properties and imbues them
// to the object instance
class Assignable {
  constructor(properties) {
    Object.keys(properties).map((key) => {
      return (this[key] = properties[key]);
    });
  }
}

// Our instruction payload vocabulary
class Payload extends Assignable {}

// Instruction variant indexes
const InstructionVariant = {
  Echo: 0,
  InitializeAuthorizedEcho: 1,
  AuthorizedEcho: 2,
  InitializeVendingMachineEcho: 3,
  VendingMachineEcho: 4,
};

// Borsh needs a schema describing the payload
const AuthorizedEchoPayload = new Map([
  [
    Payload,
    {
      kind: "struct",
      fields: [
        ["id", "u8"],
        ["data", "string"],
      ],
    },
  ],
]);

const InitializeVendingMachineEcho = new Map([
  [
    Payload,
    {
      kind: "struct",
      fields: [
        ["id", "u8"],
        ["price", "u64"],
        ["buffer_size", "u64"],
      ],
    },
  ],
]);

const VendingMachineEchoPayload = new Map([
  [
    Payload,
    {
      kind: "struct",
      fields: [
        ["id", "u8"],
        ["data", "string"],
      ],
    },
  ],
]);

module.exports = {
  Payload,
  InstructionVariant,
  AuthorizedEchoPayload,
  InitializeVendingMachineEcho,
  VendingMachineEchoPayload,
};
