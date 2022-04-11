import { Pet } from "./pet";

interface Account {
  id: number;
  name: string;
  passwordHash: string;
}

interface Profile {
  id: number;
  firstName: string;
  lastName: string;
  age: number;
}

export interface User {
  id: string;
  profile: Profile;
  account: Account;
  pets: Pet[];
}