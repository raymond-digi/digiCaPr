<template>
  <v-container>
    <!-- Delete Autofill Confirmation Dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title class="text-h6">Confirm Delete</v-card-title>
        <v-card-text>
          Are you sure you want to delete this autofill entry?
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" variant="text" @click="confirmDeleteAutofill">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
    <v-row>
      <!-- SIN -->
      <v-col cols="6" sm="4" md="3">
        <v-text-field v-model="formData.sin" label="SIN*" :rules="[rules.required, rules.sin]" variant="outlined"
          density="comfortable" placeholder="XXX-XXX-XXX" />
      </v-col>

      <!-- Date of Birth -->
      <v-col cols="6" sm="4" md="3">
        <v-text-field v-model="formData.date_of_birth" label="Date of Birth*"
          :rules="[rules.required, rules.date, rules.dateOfBirth]" variant="outlined" density="comfortable" type="date"
          hint="Needed for CPP eligibility (ages 18-70)" persistent-hint />
      </v-col>
    </v-row>

    <v-row>
      <!-- EI Exemption -->
      <v-col cols="6" sm="4" md="3">
        <v-switch v-model="formData.ei_exempt" label="EI Exempt" color="primary" density="comfortable"
          hint="Skip EI deductions" persistent-hint />
      </v-col>

      <!-- CPP Exemption -->
      <v-col cols="6" sm="4" md="3">
        <v-switch v-model="formData.cpp_exempt" label="CPP Exempt" color="primary" density="comfortable"
          hint="Skip CPP deductions" persistent-hint />
      </v-col>
    </v-row>

    <v-row>
      <!-- Pay Type -->
      <v-col cols="6" sm="4" md="3">
        <v-select v-model="formData.pay_type" label="Pay Type*" :items="payTypes" :rules="[rules.required]"
          variant="outlined" density="comfortable" />
      </v-col>

      <!-- Pay Rate -->
      <v-col cols="6" sm="4" md="3">
        <v-text-field v-model.number="formData.pay_rate" label="Pay Rate*"
          :rules="[rules.required, rules.positiveNumber]" variant="outlined" density="comfortable" type="number"
          step="0.01" :prefix="formData.pay_type === 'Hourly' ? '$' : ''"
          :suffix="formData.pay_type === 'Hourly' ? '/hour' : formData.pay_type === 'Weekly' ? '/week' : formData.pay_type === 'Monthly' ? '/month' : '/year'" />
      </v-col>

      <!-- Vacation Pay Rate -->
      <v-col cols="6" sm="4" md="3">
        <v-text-field v-model.number="vacationPayRateDisplay" label="Vacation Pay Rate*"
          :rules="[rules.required, rules.vacationRate]" variant="outlined" density="comfortable" type="number"
          step="0.1" suffix="%" hint="e.g., 4 for 4%, 6 for 6%" persistent-hint />
      </v-col>

      <!-- Overtime Multiplier -->
      <v-col cols="6" sm="4" md="3">
        <v-text-field v-model.number="formData.overtime_multiplier" label="Overtime Multiplier*"
          :rules="[rules.required, rules.overtimeMultiplier]" variant="outlined" density="comfortable" type="number"
          step="0.1" suffix="x" hint="Typically 1.5x" persistent-hint />
      </v-col>
    </v-row>

    <v-row>
      <!-- Personal Amount Management -->
      <v-col cols="12">
        <v-card variant="outlined">
          <v-card-title class="text-subtitle-1">
            Personal Amount
          </v-card-title>
          <v-card-text>
            <!-- Province of Employment and Year -->
            <v-row>
              <v-col cols="6" sm="4" md="3">
                <v-select v-model="taxYear" label="Tax Year*" :items="availableYears" :rules="[rules.required]"
                  variant="outlined" density="comfortable" />
              </v-col>
              <v-col cols="6" sm="4" md="3">
                <v-select v-model="taxProvince" label="Tax Province*" :items="provinces" :rules="[rules.required]"
                  variant="outlined" density="comfortable" />
              </v-col>
              <v-col cols="6" sm="4" md="3">
                <v-text-field v-model.number="personalAmount.federal_amount" label="Federal Amount*" type="number"
                  step="0.01" prefix="$" :rules="[rules.required, rules.positiveNumber]" variant="outlined"
                  density="comfortable" :hint="`Default: $${basicAmounts.federal_amount.toFixed(0)}`" persistent-hint />
              </v-col>
              <v-col cols="6" sm="4" md="3">
                <v-text-field v-model.number="personalAmount.provincial_amount" label="Provincial Amount*" type="number"
                  step="0.01" prefix="$" :rules="[rules.required, rules.positiveNumber]" variant="outlined"
                  density="comfortable" :hint="`Default: $${basicAmounts.provincial_amount.toFixed(0)}`"
                  persistent-hint />
              </v-col>
            </v-row>
          </v-card-text>
          <v-card-actions>
            <span class="text-caption text-grey">Only current view of Personal Amounts will be saved</span>
            <v-spacer></v-spacer>
            <v-btn @click="loadDefaultBasicAmounts" :loading="loadingPersonal" variant="outlined">
              Load Default
            </v-btn>
            <v-btn @click="indexFromPrevious" :loading="loadingPersonal" :disabled="!canIndexFromPrevious"
              variant="outlined">
              Index from Previous
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <v-row>
      <!-- Autofill Management -->
      <v-col cols="12">
        <v-card variant="outlined">
          <v-card-title class="text-subtitle-1">
            Autofill Values (Default Earnings & Deductions)
          </v-card-title>
          <v-card-text>
            <v-row>
              <v-col cols="6" sm="4" md="3">
                <v-select v-model="newAutofill.autofill_type" label="Type*" :items="autofillTypes" :rules="[rules.required]"
                  variant="outlined" density="comfortable" />
              </v-col>
              <v-col cols="6" sm="4" md="3">
                <v-autocomplete v-model="newAutofill.type_name" label="Name*" :rules="[rules.required]"
                  :items="availableTypeNames" variant="outlined" density="comfortable" clearable
                  placeholder="Select or type a name" />
              </v-col>
              <v-col cols="6" sm="4" md="3">
                <v-text-field v-model.number="newAutofill.amount" label="Amount*" type="number" step="0.01" prefix="$"
                  :rules="[rules.required, rules.positiveNumber]" variant="outlined" density="comfortable" />
              </v-col>
              <v-col cols="6" sm="4" md="3" class="d-flex align-center">
                <v-btn @click="addAutofill" color="primary" variant="outlined" :disabled="!canAddAutofill">
                  Add
                </v-btn>
              </v-col>
            </v-row>

            <v-table density="compact" v-if="autofillEntries.length > 0">
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Name</th>
                  <th>Amount</th>
                  <th>Active</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(entry, index) in autofillEntries" :key="(entry as any).id ?? (entry as any).tempId ?? index">
                  <td>
                    <v-select v-model="entry.autofill_type" :items="autofillTypes" density="compact" variant="plain"
                      hide-details @update:model-value="updateAutofill(entry)" />
                  </td>
                  <td>
                    <v-autocomplete v-model="entry.type_name" :items="getAvailableNamesForEntry(entry)" density="compact"
                      variant="plain" hide-details clearable @update:model-value="updateAutofill(entry)" />
                  </td>
                  <td>
                    <v-text-field v-model.number="entry.amount" type="number" step="0.01" prefix="$" density="compact"
                      variant="plain" hide-details @update:model-value="updateAutofill(entry)" />
                  </td>
                  <td>
                    <v-switch v-model="entry.is_active" color="primary" density="compact" hide-details
                      @change="updateAutofill(entry)" />
                  </td>
                  <td>
                    <v-btn icon="mdi-delete" size="x-small" variant="text" color="error" @click="deleteAutofill((entry as any).id ?? (entry as any).tempId)" />
                  </td>
                </tr>
              </tbody>
            </v-table>
            <div v-else class="text-caption text-grey pa-2">
              No autofill values configured. Add default earnings or deductions above.
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue'
import { DEDUCTION_TYPES, EARNING_TYPES } from '@/types/payroll'
import type { Employee, PersonalAmount, EmployeeAutofill, AutofillType } from '@/types/employee'
import { personalAmountApi, employeeApi } from '@/services/api'
import { getErrorMessage } from '@/utils/error'
const props = defineProps<{
  formData: Employee
  rules: any
  payTypes: string[]
  provinces: string[]
}>()

// Vacation pay rate displayed as percentage (stored as decimal, displayed as 0-20)
const vacationPayRateDisplay = computed({
  get: () => (props.formData.vacation_pay_rate ?? 0) * 100,
  set: (val: number) => { props.formData.vacation_pay_rate = val / 100 }
})

// Current personal amount being viewed/edited
const personalAmount = ref<PersonalAmount>({
  id: undefined,
  employee_id: 0,
  province: 'ON',
  year: new Date().getFullYear(),
  federal_amount: 0,
  provincial_amount: 0,
  indexed_at: '',
})

// In-memory collection of all personal amounts for this employee
const personalAmounts = ref<PersonalAmount[]>([])

const loadingPersonal = ref(false)
const availableYears = ref<number[]>([])
const taxProvince = ref('')
const taxYear = ref(new Date().getFullYear())
const isInitialized = ref(false)

const basicAmounts = ref({
  federal_amount: 0,
  provincial_amount: 0
})

const canIndexFromPrevious = computed(() => {
  const hasPreviousSameProvince = personalAmounts.value.some(
    pa => pa.province === taxProvince.value && pa.year !== taxYear.value
  );
  const hasZeroAmount =
    personalAmount.value.federal_amount === 0 ||
    personalAmount.value.provincial_amount === 0;
  return hasPreviousSameProvince && hasZeroAmount;
});

// Load available tax years from config files
const loadAvailableYears = async () => {
  try {
    const years = await personalAmountApi.getAvailableTaxYears()
    availableYears.value = years
  } catch (error) {
    console.error('Failed to load available tax years:', error)
    // Show error to user - no fallback allowed
    alert(`Error loading tax configuration files: ${getErrorMessage(error)}\n\nPlease ensure tax_rates_*.json files exist in the config directory.`)
    availableYears.value = []
  }
}

// Load all personal amounts for the employee into memory
const loadPersonalAmounts = async () => {
  if (!props.formData.id) {
    personalAmounts.value = []
    return
  }

  loadingPersonal.value = true
  try {
    const employeeId = props.formData.id
    const amounts = await personalAmountApi.getPersonalAmounts(employeeId)
    personalAmounts.value = amounts
    console.log(`Loaded ${amounts.length} personal amount(s) into memory`)

    // Determine optimal taxProvince and taxYear from personal amounts using priority rules
    const hireProvince = props.formData.hire_province
    const addressProvince = props.formData.address.province ?? null
    let targetProvince: string | null = null
    let targetYear: number | null = null

    if (hireProvince && personalAmounts.value.some(pa => pa.province === hireProvince)) {
      const filtered = personalAmounts.value.filter(pa => pa.province === hireProvince)
      targetYear = Math.max(...filtered.map(pa => pa.year))
      targetProvince = hireProvince
    } else if (addressProvince && personalAmounts.value.some(pa => pa.province === addressProvince)) {
      const filtered = personalAmounts.value.filter(pa => pa.province === addressProvince)
      targetYear = Math.max(...filtered.map(pa => pa.year))
      targetProvince = addressProvince
    } else if (personalAmounts.value.length > 0) {
      const latest = personalAmounts.value.reduce((prev, current) => prev.year > current.year ? prev : current)
      targetProvince = latest.province
      targetYear = latest.year
    }

    if (targetProvince && targetYear !== null) {
      taxProvince.value = targetProvince
      taxYear.value = targetYear
    } else {
      // Fallback
      if (hireProvince) {
        taxProvince.value = hireProvince
      } else if (addressProvince) {
        taxProvince.value = addressProvince
      }
      taxYear.value = new Date().getFullYear()
    }

    // Look up the current amount based on taxProvince and taxYear
    lookupPersonalAmount()
  } catch (error) {
    console.error('Failed to load personal amounts:', error)
    personalAmounts.value = []
  } finally {
    loadingPersonal.value = false
  }
}

// Look up personal amount from in-memory collection by taxProvince and taxYear
const lookupPersonalAmount = () => {
  if (!taxProvince.value || !taxYear.value) {
    console.warn('Tax province and year are required to lookup personal amounts')
    return
  }

  // Find in the in-memory collection
  const found = personalAmounts.value.find(
    pa => pa.province === taxProvince.value && pa.year === taxYear.value
  )

  if (found) {
    // Use the found personal amount from in-memory collection
    personalAmount.value = { ...found }
    // Load basic amounts for hints
    loadBasicAmounts()
  } else {
    // No match in memory - load from tax config
    personalAmount.value = {
      id: undefined,
      employee_id: props.formData.id || 0,
      province: taxProvince.value,
      year: taxYear.value,
      federal_amount: 0,
      provincial_amount: 0,
      indexed_at: '',
    }
  }
}

const loadBasicAmounts = async () => {
  if (!taxProvince.value || !taxYear.value) {
    return
  }
  try {
    const amounts = await personalAmountApi.getBasicPersonalAmounts(
      taxProvince.value,
      taxYear.value
    )
    basicAmounts.value.federal_amount = amounts.federal_amount
    basicAmounts.value.provincial_amount = amounts.provincial_amount
  } catch (error) {
    console.error('Failed to load basic personal amounts for hints:', error)
  }
}

const loadDefaultBasicAmounts = async () => {
  personalAmount.value.federal_amount = basicAmounts.value.federal_amount
  personalAmount.value.provincial_amount = basicAmounts.value.provincial_amount
}

const indexFromPrevious = async () => {
  if (!taxProvince.value || !taxYear.value) {
    alert('Please select tax province and year first.');
    return;
  }

  const others = personalAmounts.value.filter(
    pa => pa.province === taxProvince.value && pa.year !== taxYear.value
  );

  if (others.length === 0) {
    alert('No previous personal amounts found for this province.');
    return;
  }

  const previous = others.reduce((a, b) => a.year > b.year ? a : b);

  loadingPersonal.value = true;
  try {
    const prevBasics = await personalAmountApi.getBasicPersonalAmounts(
      taxProvince.value,
      previous.year
    );

    if (prevBasics.federal_amount === 0 || prevBasics.provincial_amount === 0) {
      throw new Error('Invalid previous basic amounts (zero values)');
    }

    const fedIndex = basicAmounts.value.federal_amount / prevBasics.federal_amount;
    const provIndex = basicAmounts.value.provincial_amount / prevBasics.provincial_amount;

    personalAmount.value.federal_amount = Math.round(previous.federal_amount * fedIndex * 100) / 100;
    personalAmount.value.provincial_amount = Math.round(previous.provincial_amount * provIndex * 100) / 100;

    personalAmount.value.indexed_at = new Date().toISOString();
  } catch (error) {
    console.error('Failed to index personal amounts:', error);
    alert(`Failed to index amounts: ${error instanceof Error ? error.message : 'Unknown error'}`);
  } finally {
    loadingPersonal.value = false;
  }
};

const savePersonalAmount = async () => {
  if (!props.formData.id) {
    return
  }

  // Update the current personal amount with the current values
  personalAmount.value.employee_id = props.formData.id
  personalAmount.value.province = taxProvince.value
  personalAmount.value.year = taxYear.value
  personalAmount.value.indexed_at = new Date().toISOString()

  try {
    if (personalAmount.value.id !== undefined) {
      // Update existing record
      await personalAmountApi.updatePersonalAmount(personalAmount.value)
      // Update in-memory collection
      const index = personalAmounts.value.findIndex(
        pa => pa.id === personalAmount.value.id
      )
      if (index !== -1) {
        personalAmounts.value[index] = { ...personalAmount.value }
      }
    } else {
      // Create new record
      const newId = await personalAmountApi.createPersonalAmount(personalAmount.value)
      personalAmount.value.id = newId
      // Add to in-memory collection
      personalAmounts.value.push({ ...personalAmount.value })
    }
    console.log('Personal amount saved successfully')
  } catch (error) {
    console.error('Failed to save personal amount:', error)
    throw error // Re-throw to let parent handle error
  }
}

// Autofill management
interface TempAutofill extends EmployeeAutofill {
  tempId: string
}

const autofillEntries = ref<(EmployeeAutofill | TempAutofill)[]>([])

// Delete confirmation state
const deleteDialog = ref(false)
const itemToDelete = ref<{ identifier: number | string; index: number } | null>(null)

const pendingNewTempIds = ref<string[]>([])
const pendingDirtyIds = ref<number[]>([])
const pendingDeleteIds = ref<number[]>([])

const newAutofill = ref<Partial<EmployeeAutofill>>({
  autofill_type: 'earning' as AutofillType,
  type_name: '',
  amount: 0,
  is_active: true
})

const autofillTypes = [
  { title: 'Earning', value: 'earning' },
  { title: 'Deduction', value: 'deduction' }
]

// Get available names based on selected autofill type
const availableTypeNames = computed(() => {
  if (newAutofill.value.autofill_type === 'earning') {
    return EARNING_TYPES.map(et => et.name)
  } else if (newAutofill.value.autofill_type === 'deduction') {
    return DEDUCTION_TYPES.map(dt => dt.name)
  }
  return []
})

// Get available names for an existing autofill entry
const getAvailableNamesForEntry = (entry: EmployeeAutofill) => {
  if (entry.autofill_type === 'earning') {
    return EARNING_TYPES.map(et => et.name)
  } else if (entry.autofill_type === 'deduction') {
    return DEDUCTION_TYPES.map(dt => dt.name)
  }
  return []
}

const canAddAutofill = computed(() => {
  return newAutofill.value.type_name &&
         newAutofill.value.type_name.trim() !== '' &&
         (newAutofill.value.amount || 0) >= 0 &&
         newAutofill.value.autofill_type
})

const loadAutofills = async () => {
  if (!props.formData.id) {
    autofillEntries.value = []
    pendingNewTempIds.value = []
    pendingDirtyIds.value = []
    pendingDeleteIds.value = []
    return
  }

  try {
    const entries = await employeeApi.getEmployeeAutofill(props.formData.id)
    autofillEntries.value = entries
    pendingNewTempIds.value = []
    pendingDirtyIds.value = []
    pendingDeleteIds.value = []
  } catch (error) {
    console.error('Failed to load autofill entries:', error)
    autofillEntries.value = []
    pendingNewTempIds.value = []
    pendingDirtyIds.value = []
    pendingDeleteIds.value = []
  }
}

const addAutofill = () => {
  if (!canAddAutofill.value) {
    return
  }

  const tempId = `new-${Date.now()}`
  const autofill: EmployeeAutofill & { tempId: string } = {
    id: undefined,
    employee_id: props.formData.id || 0,
    autofill_type: newAutofill.value.autofill_type! as AutofillType,
    type_name: newAutofill.value.type_name!,
    amount: newAutofill.value.amount || 0,
    is_active: true,
    tempId
  }

  pendingNewTempIds.value.push(tempId)
  autofillEntries.value.push(autofill)

  // Reset form
  newAutofill.value = {
    autofill_type: 'earning' as AutofillType,
    type_name: '',
    amount: 0,
    is_active: true
  }
}

const updateAutofill = (entry: any) => {
  if (entry.tempId) {
    if (!pendingNewTempIds.value.includes(entry.tempId)) {
      pendingNewTempIds.value.push(entry.tempId)
    }
  } else if (entry.id !== undefined) {
    if (!pendingDirtyIds.value.includes(entry.id)) {
      pendingDirtyIds.value.push(entry.id)
    }
  }
}

const deleteAutofill = (identifier: number | string) => {
  const entryIndex = autofillEntries.value.findIndex((e: EmployeeAutofill | TempAutofill) =>
    e.id === identifier || (e as TempAutofill).tempId === identifier
  )
  if (entryIndex === -1) {
    return
  }

  // Store the item to delete and show confirmation dialog
  itemToDelete.value = { identifier, index: entryIndex }
  deleteDialog.value = true
}

const confirmDeleteAutofill = () => {
  if (!itemToDelete.value) {
    return
  }

  const { index: entryIndex } = itemToDelete.value
  const entry = autofillEntries.value[entryIndex] as EmployeeAutofill | TempAutofill

  if (entry.id !== undefined) {
    pendingDeleteIds.value.push(entry.id)
  } else if ((entry as TempAutofill).tempId) {
    const tempIdx = pendingNewTempIds.value.indexOf((entry as TempAutofill).tempId)
    if (tempIdx !== -1) {
      pendingNewTempIds.value.splice(tempIdx, 1)
    }
  }

  autofillEntries.value.splice(entryIndex, 1)
  deleteDialog.value = false
  itemToDelete.value = null
}

const saveAutofills = async () => {
  if (!props.formData.id) {
    return
  }

  try {
    // Process deletes first
    for (const delId of pendingDeleteIds.value) {
      await employeeApi.deleteEmployeeAutofill(delId)
    }

    // Process updates
    for (const dirtyId of pendingDirtyIds.value) {
      const entry = autofillEntries.value.find((e: any) => e.id === dirtyId)
      if (entry) {
        await employeeApi.saveEmployeeAutofill(entry)
      }
    }

    // Process creates
    for (const tempId of pendingNewTempIds.value) {
      const entry = autofillEntries.value.find((e: EmployeeAutofill | TempAutofill) => (e as TempAutofill).tempId === tempId) as TempAutofill
      if (entry) {
        entry.employee_id = props.formData.id!
        const newId = await employeeApi.saveEmployeeAutofill(entry as EmployeeAutofill)
        entry.id = newId
        delete (entry as any).tempId
      }
    }

    // Clear pending
    pendingDeleteIds.value = []
    pendingDirtyIds.value = []
    pendingNewTempIds.value = []

    // Reload to ensure sync
    await loadAutofills()
  } catch (error) {
    console.error('Failed to save autofills:', error)
    throw error
  }
}

onMounted(() => {
  loadAvailableYears()
})

watch(
  () => props.formData.id,
  (newId) => {
    isInitialized.value = false
    if (newId) {
      // Only load when employee ID is actually set (editing mode)
      loadPersonalAmounts()
      loadAutofills()
      isInitialized.value = true
    } else {
      autofillEntries.value = []
      pendingNewTempIds.value = []
      pendingDirtyIds.value = []
      pendingDeleteIds.value = []
    }
  },
  { immediate: true }
)

watch(
  () => taxProvince.value,
  () => {
    loadBasicAmounts()
    // Look up the amount in memory when province changes
    if (isInitialized.value) {
      lookupPersonalAmount()
    }
  }
)

watch(
  () => taxYear.value,
  () => {
    loadBasicAmounts()
    // Look up the amount in memory when year changes
    if (isInitialized.value) {
      lookupPersonalAmount()
    }
  }
)

defineExpose({
  savePersonalAmount,
  saveAutofills
})
</script>